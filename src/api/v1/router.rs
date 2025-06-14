use super::error::*;
use super::handler;
use crate::auth::*;
use crate::chat::ChatService;
use crate::server::*;
use std::convert::Infallible;
use std::sync::Arc;
use warp::{http, reject, Filter};
use crate::domain::UserId;

pub fn routes(
    server: Server,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let captcha = warp::get()
        .and(warp::path("captcha"))
        .and(warp::path::end())
        .and(with(server.captcha_service.clone()))
        .and_then(handler::generate_captcha);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with(server.auth_service.clone()))
        .and(with(server.captcha_service.clone()))
        .and_then(handler::login);

    let signup = warp::post()
        .and(warp::path("signup"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with(server.auth_service.clone()))
        .and(with(server.captcha_service.clone()))
        .and_then(handler::signup);

    let chat = warp::get()
        .and(warp::path("chat"))
        .and(warp::path::end())
        .and(with_verification(server.auth_service.clone()))
        .and(warp::ws())
        .and(with(server.chat_service.clone()))
        .map(
            |user_id: UserId, ws: warp::ws::Ws, chat_service: Arc<dyn ChatService>| {
                ws.on_upgrade(|socket| handler::join_chat(socket, user_id, chat_service))
            },
        );

    captcha.or(login).or(signup).or(chat)
}

fn with<ServiceType>(
    service: Arc<ServiceType>,
) -> impl Filter<Extract = (Arc<ServiceType>,), Error = Infallible> + Clone
where
    ServiceType: Send + Sync + ?Sized,
{
    warp::any().map(move || service.clone())
}

fn with_verification(
    auth_service: Arc<dyn AuthService>,
) -> impl Filter<Extract = (UserId,), Error = warp::Rejection> + Clone {
    warp::header::<String>(http::header::AUTHORIZATION.as_ref()).and_then(move |token: String| {
        let auth_service = auth_service.clone();
        async move {
            if let Some(token) = token.strip_prefix("Bearer ") {
                let user_id = auth_service
                    .verify_token(token)
                    .await
                    .map_err(map_auth_error_to_api_error)
                    .map_err(reject::custom)?;
                Ok(user_id)
            } else {
                Err(reject::custom(ApiError::InvalidToken))
            }
        }
    })
}
