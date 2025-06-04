use futures_util::sink::SinkExt;
use anyhow::{Result, anyhow};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use captcha_rs::CaptchaBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use futures_util::StreamExt;
use jsonwebtoken::{DecodingKey, Validation};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, trace, warn};
use uuid::Uuid;
use warp::{http, Filter, Rejection};
use warp::ws::Message;

#[derive(Debug)]
struct User {
    hashed_password: String,
    connection: Option<tokio::sync::mpsc::UnboundedSender<Message>>,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
    captcha_id: String,
    captcha_answer: String,
}

#[derive(Debug, Serialize)]
struct CaptchaResponse {
    id: String,
    image_base64: String,
}

// region login

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
}

async fn verify_password(user_store: UserStore, username: &str, password: &str) -> bool {
    let mut map = user_store.read().await;
    if let Some(User{hashed_password: stored_hash, .. }) = map.get(&username.to_string()) {
        let parsed_hash = PasswordHash::new(stored_hash);
        return match parsed_hash {
            Ok(hash) => Argon2::default().verify_password(password.as_bytes(), &hash).is_ok(),
            Err(_) => false,
        }
    }
    false
}

async fn generate_jwt(username: &str) -> Result<String> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(1))
        .expect("valid timestamp")
        .timestamp() as u64;

    let claims = Claims {
        sub: username.to_string(),
        exp: expiration
    };

    let secret = std::env::var("CAPTCHA_SECRET").unwrap_or_else(|_| "my_secret".into());

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes())
    ).map_err(|e| anyhow!(e.to_string()))
}

async fn login_handler(body: LoginRequest, captcha_store: CaptchaStore, user_store: UserStore) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("login_handler: {:?}", body);

    if !verify_captcha(captcha_store, body.captcha_id, body.captcha_answer).await {
        return Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "error": "invalid captcha"
        })));
    }

    if !verify_password(user_store, &body.username, &body.password).await {
        return Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "error": "invalid username or password"
        })))
    }

    if let Ok(jwt) = generate_jwt(&body.username).await {
        Ok(warp::reply::json(
            &serde_json::json!({"token": jwt}),
        ))
    } else {
        Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "error": "internal server error"
        })))
    }
}

// endregion

//region signup

#[derive(Debug, Deserialize)]
struct SignupRequest {
    username: String,
    password: String,
    captcha_id: String,
    captcha_answer: String,
}

async fn signup_handler(
    body: SignupRequest,
    captcha_store: CaptchaStore,
    user_store: UserStore,
) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("signup_handler: {:?}", body);

    if !verify_captcha(captcha_store, body.captcha_id, body.captcha_answer).await {
        return Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "error": "invalid captcha"
        })));
    }

    if let Err(e) = validate_password(&body.password) {
        return Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "error": e.to_string()
        })));
    }

    let hashed = match hash_password(&body.password) {
        Ok(h) => h,
        Err(_) => {
            return Ok(warp::reply::json(&serde_json::json!({
                "status": "error",
                "error": "internal server error",
            })));
        }
    };

    let mut map = user_store.write().await;
    if map.contains_key(&body.username) {
        return Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "error": "user already exists",
        })));
    }

    map.insert(
        body.username.clone(),
        User {
            hashed_password: hashed,
            connection: None,
        },
    );

    Ok(warp::reply::json(&serde_json::json!({
        "status": "signup successful"
    })))
}

fn validate_password(password: &str) -> Result<()> {
    if password.len() < 8 {
        Err(anyhow!("password too short"))
    } else {
        Ok(())
    }
}

fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = argon2::Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!(e))?;

    Ok(password_hash.to_string())
}

//endregion

// region captcha

async fn captcha_handler(store: CaptchaStore) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("captcha_handler");

    let (id, image_base64) = generate_captcha(store).await;

    let response = CaptchaResponse { id, image_base64 };
    Ok(warp::reply::json(&response))
}

// DEV NOTE: Failed or time out captchas are never cleaned up. This is acceptable during development.
// TODO: Replace with TTL store (Redis) for production.
async fn generate_captcha(store: CaptchaStore) -> (String, String) {
    let captcha = CaptchaBuilder::new()
        .length(6)
        .width(100)
        .height(50)
        .dark_mode(true)
        .complexity(1)
        .build();

    let id = Uuid::new_v4().to_string();
    let image_base64 = captcha
        .to_base64()
        .strip_prefix("data:image/jpeg;base64,")
        .unwrap()
        .to_string();
    {
        let mut map = store.lock().await;
        map.insert(id.clone(), captcha.text.clone());
    }
    (id, image_base64)
}

async fn verify_captcha(store: CaptchaStore, id: String, answer: String) -> bool {
    trace!("verify_captcha");

    let mut map = store.lock().await;
    if let Some(expected) = map.remove(&id) {
        return expected.eq_ignore_ascii_case(&answer);
    }
    false
}

// endregion

type UserStore = Arc<RwLock<HashMap<String, User>>>;
type CaptchaStore = Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("warp=off,server_oxide=trace")
        .init();

    let user_store: UserStore = Arc::new(RwLock::new(HashMap::new()));
    let captcha_store: CaptchaStore = Arc::new(Mutex::new(HashMap::new()));

    let hello = warp::path::end().map(|| "Hello, HTTPS!\n");

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_captcha_store(captcha_store.clone()))
        .and(with_user_store(user_store.clone()))
        .and_then(login_handler);

    let signup = warp::post()
        .and(warp::path("signup"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_captcha_store(captcha_store.clone()))
        .and(with_user_store(user_store.clone()))
        .and_then(signup_handler);

    let captcha = warp::get()
        .and(warp::path("captcha"))
        .and(warp::path::end())
        .and(with_captcha_store(captcha_store.clone()))
        .and_then(captcha_handler);

    let cert_path = "cert/cert.pem";
    let key_path = "cert/key.pem";

    let chat = warp::get()
        .and(warp::path("chat"))
        .and(warp::path::end())
        .and(with_auth())
        .and(warp::ws())
        .and(with_user_store(user_store.clone()))  // Avoid unnecessary clone
        .map(|username: String, ws: warp::ws::Ws, user_store: UserStore| {
            ws.on_upgrade(|socket| user_connected(socket, username, user_store))
        });

    let routes = hello.or(login).or(signup).or(captcha).or(chat);

    warp::serve(routes)
        .tls()
        .cert_path(cert_path)
        .key_path(key_path)
        .run(([127, 0, 0, 1], 8443))
        .await;
}

async fn user_connected(mut socket: warp::ws::WebSocket, username: String, user_store: UserStore) {
    trace!("user connected: {}", username);

    let (mut to_user, mut from_user) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    if let Some(user) = user_store.write().await.get_mut(&username) {
        user.connection = Some(tx);
    }

    let to_user_handle = tokio::task::spawn(async move {
        while let Some(message) = rx.recv().await {
            trace!("sending websocket message: {:?}", message);
            if let Err(e) = to_user.send(message).await {
                warn!("failed to send websocket message: {:?}", e);
                break;
            }
        }
    });

    let username_clone = username.clone();
    let user_store_clone = user_store.clone();
    let from_user_handle = tokio::task::spawn(async move {
        while let Some(result) = from_user.next().await {
            trace!("websocket received: {:?}", result);
            if let Ok(message) = result {
                let message = format!("<{}>: {}", username_clone, message.to_str().unwrap_or_default());
                for User{connection: conn, .. } in user_store_clone.read().await.values() {
                    if let Some(tx) = conn {
                        trace!("sending websocket message to: {:?}", username_clone);
                        if let Err(e) = tx.send(Message::text(message.clone())) {
                            info!("user disconnected: {}", username_clone);
                        }
                    } else {
                        trace!("user offline: {}", username_clone);
                    }
                }
            } else {
                warn!("websocket received error: {:?}", result);
                break;
            }
        }
    });

    let _ = tokio::try_join!(to_user_handle, from_user_handle);

    user_store.write().await.get_mut(&username).unwrap().connection = None;
    trace!("user disconnected: {}", username);
}

fn with_user_store(
    store: UserStore,
) -> impl Filter<Extract = (UserStore,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || store.clone())
}

fn with_captcha_store(
    store: CaptchaStore,
) -> impl Filter<Extract = (CaptchaStore,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || store.clone())
}

fn with_auth() -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::header::<String>(http::header::AUTHORIZATION.as_ref()).and_then(|auth: String| async move {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            trace!("auth token: {}", token);

            let secret = std::env::var("CAPTCHA_SECRET").unwrap_or_else(|_| "my_secret".into());
            let token_data = jsonwebtoken::decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &Validation::default(),
            );

            return match token_data {
                Ok(data) => {
                    trace!("token decoded successfully");
                    Ok(data.claims.sub.clone())
                }
                Err(e) => {
                    warn!("token decode error: {}", e);
                    Err(warp::reject())
                }
            }
        } else {
            warn!("auth failed");
            Err(warp::reject())
        }
    })
}
