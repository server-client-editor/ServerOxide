use anyhow::{Result, anyhow};
use argon2::PasswordHasher;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::trace;
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
    question: String,
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
    captcha: String,
}

async fn signup_handler(
    body: SignupRequest,
    users: Arc<RwLock<HashMap<String, User>>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("signup_handler: {:?}", body);

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

async fn captcha_handler() -> Result<impl warp::Reply, warp::Rejection> {
    trace!("captcha_handler");
    let response = CaptchaResponse {
        id: "3".to_string(),
        question: "What is 1 + 2?".to_string(),
    };
    Ok(warp::reply::json(&response))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("warp=off,ServerOxide=trace")
        .init();

    let users: Arc<RwLock<HashMap<String, User>>> = Arc::new(RwLock::new(HashMap::new()));
    let user_filter = warp::any().map(move || users.clone());

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
        .and_then(signup_handler);

    let captcha = warp::get()
        .and(warp::path("captcha"))
        .and(warp::path::end())
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
