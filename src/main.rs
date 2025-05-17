use serde::{Deserialize, Serialize};
use tracing::trace;
use warp::Filter;

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
    captcha: String,
}

#[derive(Debug, Deserialize)]
struct SignupRequest {
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

async fn signup_handler(body: SignupRequest) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("signup_handler: {:?}", body);
    Ok(warp::reply::json(
        &serde_json::json!({"status": "signup successful"}),
    ))
}

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
