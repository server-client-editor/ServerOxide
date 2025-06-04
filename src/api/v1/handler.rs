use futures_util::sink::SinkExt;
use warp;

pub async fn generate_captcha() -> Result<impl warp::Reply, warp::Rejection> {
    Ok("Generate captcha\n")
}
pub async fn login(body: serde_json::Value) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(format!("Login ({})\n", body))
}
pub async fn signup(body: serde_json::Value) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(format!("Signup ({})\n", body))
}
pub async fn join_chat(mut socket: warp::ws::WebSocket) {
    let greeting = "Hello from WebSocket!\n".to_string();
    let _ = socket.send(warp::ws::Message::text(greeting)).await;
    let _ = socket.close();
}