pub mod auth;
pub mod markets;
pub mod trading;
pub mod api;

use crate::Database;
use crate::repository::UserRepository;
use crate::web::session::OptionalAuth;
use askama::Template;
use axum::{response::Html, extract::State};

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    username: Option<String>,
}

pub async fn home(
    auth: OptionalAuth,
    State(db): State<Database>,
) -> Html<String> {
    let username = if let Some(user_id) = auth.user_id {
        let user_repo = UserRepository::new(db.pool().clone());
        user_repo.find_by_id(user_id)
            .await
            .ok()
            .map(|u| u.username)
    } else {
        None
    };

    let template = HomeTemplate { username };
    Html(template.render().unwrap())
}
