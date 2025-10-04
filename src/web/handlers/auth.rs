use crate::Database;
use crate::repository::UserRepository;
use crate::web::session::{set_user_session, clear_user_session};
use axum::{
    extract::State,
    response::{Html, Redirect},
    Form,
};
use askama::Template;
use serde::Deserialize;
use tower_sessions::Session;

#[derive(Template)]
#[template(path = "signup.html")]
struct SignupTemplate {
    error: Option<String>,
    username: Option<String>,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    error: Option<String>,
    username: Option<String>,
}

#[derive(Deserialize)]
pub struct SignupForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub async fn signup_page() -> Html<String> {
    let template = SignupTemplate {
        error: None,
        username: None,
    };
    Html(template.render().unwrap())
}

pub async fn signup(
    State(db): State<Database>,
    Form(form): Form<SignupForm>,
) -> Result<Redirect, Html<String>> {
    // Validate input
    if form.username.is_empty() || form.password.is_empty() {
        let template = SignupTemplate {
            error: Some("Username and password are required".to_string()),
            username: None,
        };
        return Err(Html(template.render().unwrap()));
    }

    if form.username.len() < 3 {
        let template = SignupTemplate {
            error: Some("Username must be at least 3 characters".to_string()),
            username: None,
        };
        return Err(Html(template.render().unwrap()));
    }

    if form.password.len() < 6 {
        let template = SignupTemplate {
            error: Some("Password must be at least 6 characters".to_string()),
            username: None,
        };
        return Err(Html(template.render().unwrap()));
    }

    // Hash password
    let password_hash = bcrypt::hash(&form.password, bcrypt::DEFAULT_COST)
        .map_err(|_| {
            let template = SignupTemplate {
                error: Some("Error processing password".to_string()),
                username: None,
            };
            Html(template.render().unwrap())
        })?;

    // Create user
    let user_repo = UserRepository::new(db.pool().clone());
    match user_repo.create(&form.username, &password_hash).await {
        Ok(_) => Ok(Redirect::to("/login")),
        Err(e) => {
            let template = SignupTemplate {
                error: Some(format!("Error creating account: {}", e)),
                username: None,
            };
            Err(Html(template.render().unwrap()))
        }
    }
}

pub async fn login_page() -> Html<String> {
    let template = LoginTemplate {
        error: None,
        username: None,
    };
    Html(template.render().unwrap())
}

pub async fn login(
    State(db): State<Database>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> Result<Redirect, Html<String>> {
    let user_repo = UserRepository::new(db.pool().clone());

    // Find user
    let user = match user_repo.find_by_username(&form.username).await {
        Ok(user) => user,
        Err(_) => {
            let template = LoginTemplate {
                error: Some("Invalid username or password".to_string()),
                username: None,
            };
            return Err(Html(template.render().unwrap()));
        }
    };

    // Verify password
    let password_valid = bcrypt::verify(&form.password, &user.password_hash)
        .unwrap_or(false);

    if !password_valid {
        let template = LoginTemplate {
            error: Some("Invalid username or password".to_string()),
            username: None,
        };
        return Err(Html(template.render().unwrap()));
    }

    // Create session
    set_user_session(&session, user.id).await.map_err(|_| {
        let template = LoginTemplate {
            error: Some("Error creating session".to_string()),
            username: None,
        };
        Html(template.render().unwrap())
    })?;

    Ok(Redirect::to("/markets"))
}

pub async fn logout(session: Session) -> Redirect {
    // Clear session
    let _ = clear_user_session(&session).await;
    Redirect::to("/")
}
