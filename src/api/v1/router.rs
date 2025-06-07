use crate::server::*;
use super::handler;
use warp::{Filter};

pub fn routes(server: Server) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let captcha = warp::get()
        .and(warp::path("captcha"))
        .and(warp::path::end())
        .and_then(handler::generate_captcha);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and_then(handler::login);

    let signup = warp::post()
        .and(warp::path("signup"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and_then(handler::signup);

    let chat = warp::get()
        .and(warp::path("chat"))
        .and(warp::path::end())
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(|socket| handler::join_chat(socket))
        });

    captcha.or(login).or(signup).or(chat)
}