use axum::{extract::State, response::IntoResponse};
use crate::{endpoints::{models::auth_models::refresh_token, request_json_validator::ValidJson}, services::auth_service::{AuthError, LoginError, LoginRequest}};

use crate::services::user_service::CreateError;
use crate::{
    dependency_injection::AppState,
    endpoints::{response_utils::*,
    models::auth_models::{signup, login}}
};

pub async fn signup(
    State(state): State<AppState>,
    ValidJson(request): ValidJson<signup::Request>
 ) -> impl IntoResponse {

    // TODO: validate request, use response_bad_request to return the 400 error with proper message

    let Some(currency) = state.currency_service.try_get(request.currency_id) else {
        return response_bad_request(&format!("Currency not found with ID={}", request.currency_id));
    };

    /* let currency = match state.currency_service.try_get(request.currency_id) {
        Some(c) => c,
        None => return response_bad_request(&format!("Currency not found with ID={}", request.currency_id))
    }; */

    match state.auth_service.signup(request.username, request.password, currency).await {
        Ok(_) => response_ok(signup::Response::success()),
        Err(CreateError::UsernameAlreadyInUse) => response_ok(signup::Response::error( "Username already taken")),
        Err(CreateError::DatabaseError(e)) => {
            // TODO: log error
            response_error(&e)
        }
    }
}

pub async fn login(
    State(state): State<AppState>,
    ValidJson(request): ValidJson<login::Request>
) -> impl IntoResponse {

    let username = request.username.trim().to_string();
    let password = request.password.trim().to_string();

    // TODO: use HTTP headers
    let ip_address:String = String::from("");
    let user_agent:String = String::from("");

    let service_request = LoginRequest { username:username, password:password, ip_address:ip_address, user_agent:user_agent} ;

    match state.auth_service.login(service_request).await {
        Ok(session) => response_ok( login::Response::from(session)),
        Err(LoginError::FailedLogin) => response_unhautorized("Wrong username or password"),
        Err(LoginError::DatabaseError(e)) => response_error(&e)
    }
}

pub async fn refresh_token(
    State(state): State<AppState>,
    ValidJson(request): ValidJson<refresh_token::Request>
) -> impl IntoResponse {
    match state.auth_service.refresh_session(request.token).await {
        Ok(session) => response_ok(refresh_token::Response::from(session)),
        Err(AuthError::InvalidOrExpiredToken) => response_invalid_token("Refresh token is invalid or expired."),
        Err(AuthError::DatabaseError(e)) => response_error(&e)
    }
}
