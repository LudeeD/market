use crate::domain::UserId;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response, Redirect},
};
use tower_sessions::Session;
use serde::{Deserialize, Serialize};

const SESSION_USER_ID_KEY: &str = "user_id";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub user_id: UserId,
}

impl AuthSession {
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }
}

/// Helper to set the current user in the session
pub async fn set_user_session(session: &Session, user_id: UserId) -> Result<(), tower_sessions::session::Error> {
    session.insert(SESSION_USER_ID_KEY, user_id).await
}

/// Helper to get the current user from the session
pub async fn get_user_session(session: &Session) -> Option<UserId> {
    session.get::<UserId>(SESSION_USER_ID_KEY).await.ok().flatten()
}

/// Helper to clear the session (logout)
pub async fn clear_user_session(session: &Session) -> Result<(), tower_sessions::session::Error> {
    session.remove::<UserId>(SESSION_USER_ID_KEY).await?;
    session.flush().await
}

/// Extractor that requires authentication
pub struct RequireAuth {
    pub user_id: UserId,
}

#[async_trait]
impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response()
            })?;

        match get_user_session(&session).await {
            Some(user_id) => Ok(RequireAuth { user_id }),
            None => Err(Redirect::to("/login").into_response()),
        }
    }
}

/// Extractor for optional authentication
pub struct OptionalAuth {
    pub user_id: Option<UserId>,
}

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuth
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response()
            })?;

        Ok(OptionalAuth {
            user_id: get_user_session(&session).await,
        })
    }
}
