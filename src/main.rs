use anyhow::{Result, anyhow};
use argon2::PasswordHasher;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use captcha_rs::CaptchaBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::trace;
use uuid::Uuid;
use warp::Filter;

#[derive(Debug)]
struct User {
    username: String,
    hashed_password: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
    captcha: String,
}

#[derive(Debug, Serialize)]
struct CaptchaResponse {
    id: String,
    image_base64: String,
}

async fn login_handler(body: LoginRequest) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("login_handler: {:?}", body);
    Ok(warp::reply::json(
        &serde_json::json!({"token": "fake-jwt-token"}),
    ))
}

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
    users: Arc<RwLock<HashMap<String, User>>>,
    captcha_store: CaptchaStore,
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

    let mut map = users.write().await;
    if map.contains_key(&body.username) {
        return Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "error": "user already exists",
        })));
    }

    map.insert(
        body.username.clone(),
        User {
            username: body.username,
            hashed_password: hashed,
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

type CaptchaStore = Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("warp=off,ServerOxide=trace")
        .init();

    let users: Arc<RwLock<HashMap<String, User>>> = Arc::new(RwLock::new(HashMap::new()));
    let user_filter = warp::any().map(move || users.clone());

    let captcha_store: CaptchaStore = Arc::new(Mutex::new(HashMap::new()));

    let hello = warp::path::end().map(|| "Hello, HTTPS!\n");

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and_then(login_handler);

    let signup = warp::post()
        .and(warp::path("signup"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(user_filter)
        .and(with_captcha_store(captcha_store.clone()))
        .and_then(signup_handler);

    let captcha = warp::get()
        .and(warp::path("captcha"))
        .and(warp::path::end())
        .and(with_captcha_store(captcha_store.clone()))
        .and_then(captcha_handler);

    let cert_path = "cert/cert.pem";
    let key_path = "cert/key.pem";

    let routes = hello.or(login).or(signup).or(captcha);

    warp::serve(routes)
        .tls()
        .cert_path(cert_path)
        .key_path(key_path)
        .run(([127, 0, 0, 1], 8443))
        .await;
}

fn with_captcha_store(
    store: CaptchaStore,
) -> impl Filter<Extract = (CaptchaStore,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || store.clone())
}
