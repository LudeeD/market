pub mod handlers;
pub mod middleware;
pub mod filters;
pub mod session;

use crate::Database;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{trace::TraceLayer, services::ServeDir};

pub fn create_router() -> Router<Database> {
    Router::new()
        .route("/", get(handlers::home))
        .route("/signup", get(handlers::auth::signup_page).post(handlers::auth::signup))
        .route("/login", get(handlers::auth::login_page).post(handlers::auth::login))
        .route("/logout", post(handlers::auth::logout))
        .route("/markets", get(handlers::markets::list_markets))
        .route("/markets/new", get(handlers::markets::new_market_page).post(handlers::markets::create_market))
        .route("/markets/:id", get(handlers::markets::view_market))
        .route("/markets/:id/resolve", post(handlers::markets::resolve_market))
        .route("/trade/:market_id/buy", post(handlers::trading::buy_shares))
        .route("/trade/:market_id/sell", post(handlers::trading::sell_shares))
        .route("/positions", get(handlers::trading::view_positions))
        .route("/api/markets/:market_id/price-history", get(handlers::api::get_price_history))
        .route("/api/markets/:market_id/calculate-cost", get(handlers::api::calculate_buy_cost))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
}
