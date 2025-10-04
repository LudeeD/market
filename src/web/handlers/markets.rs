use crate::Database;
use crate::repository::{MarketRepository, UserRepository, PositionRepository};
use crate::domain::{LmsrPricing, MarketSide};
use crate::web::filters;
use crate::web::session::{RequireAuth, OptionalAuth};
use axum::{
    extract::{State, Path},
    response::{Html, Redirect},
    Form,
};
use askama::Template;
use serde::Deserialize;
use chrono::{Utc, Duration};

#[derive(Template)]
#[template(path = "markets.html")]
struct MarketsTemplate {
    markets: Vec<MarketDisplay>,
    username: Option<String>,
}

#[derive(Template)]
#[template(path = "new_market.html")]
struct NewMarketTemplate {
    error: Option<String>,
    username: Option<String>,
}

#[derive(Template)]
#[template(path = "market_detail.html")]
struct MarketDetailTemplate {
    market: MarketDisplay,
    can_resolve: bool,
    username: Option<String>,
    user_positions: Vec<UserPosition>,
}

struct UserPosition {
    side: String,
    shares: f64,
    avg_price: f64,
}

struct MarketDisplay {
    id: i64,
    question: String,
    description: Option<String>,
    end_date: String,
    yes_probability: f64,
    no_probability: f64,
    total_liquidity: f64,
    resolved: bool,
    outcome: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateMarketForm {
    question: String,
    description: String,
    days_until_end: i64,
    oracle_username: Option<String>,
}

#[derive(Deserialize)]
pub struct ResolveMarketForm {
    outcome: String,
}

pub async fn list_markets(
    auth: OptionalAuth,
    State(db): State<Database>,
) -> Html<String> {
    let market_repo = MarketRepository::new(db.pool().clone());

    let username = if let Some(user_id) = auth.user_id {
        let user_repo = UserRepository::new(db.pool().clone());
        user_repo.find_by_id(user_id)
            .await
            .ok()
            .map(|u| u.username)
    } else {
        None
    };

    let markets = market_repo.list_all().await.unwrap_or_default();
    let markets_display: Vec<MarketDisplay> = markets
        .into_iter()
        .map(|m| {
            let yes_prob = LmsrPricing::implied_probability(m.q_yes, m.q_no, m.liquidity_param);
            let total_liquidity = m.total_liquidity();
            MarketDisplay {
                id: m.id,
                question: m.question,
                description: m.description,
                end_date: m.end_date.format("%Y-%m-%d %H:%M").to_string(),
                yes_probability: yes_prob * 100.0,
                no_probability: (1.0 - yes_prob) * 100.0,
                total_liquidity,
                resolved: m.resolved,
                outcome: m.outcome,
            }
        })
        .collect();

    let template = MarketsTemplate {
        markets: markets_display,
        username,
    };
    Html(template.render().unwrap())
}

pub async fn new_market_page(
    auth: RequireAuth,
    State(db): State<Database>,
) -> Html<String> {
    let username = {
        let user_repo = UserRepository::new(db.pool().clone());
        user_repo.find_by_id(auth.user_id)
            .await
            .ok()
            .map(|u| u.username)
    };

    let template = NewMarketTemplate {
        error: None,
        username,
    };
    Html(template.render().unwrap())
}

pub async fn create_market(
    auth: RequireAuth,
    State(db): State<Database>,
    Form(form): Form<CreateMarketForm>,
) -> Result<Redirect, Html<String>> {
    let username = {
        let user_repo = UserRepository::new(db.pool().clone());
        user_repo.find_by_id(auth.user_id)
            .await
            .ok()
            .map(|u| u.username)
    };

    if form.question.is_empty() {
        let template = NewMarketTemplate {
            error: Some("Question is required".to_string()),
            username,
        };
        return Err(Html(template.render().unwrap()));
    }

    if form.days_until_end < 1 {
        let template = NewMarketTemplate {
            error: Some("Market must be open for at least 1 day".to_string()),
            username,
        };
        return Err(Html(template.render().unwrap()));
    }

    let end_date = Utc::now() + Duration::days(form.days_until_end);
    let description = if form.description.is_empty() {
        None
    } else {
        Some(form.description.as_str())
    };

    let market_repo = MarketRepository::new(db.pool().clone());
    let user_repo = UserRepository::new(db.pool().clone());

    let creator_id = auth.user_id;

    // Lookup oracle by username if provided
    let oracle_id = if let Some(ref oracle_username) = form.oracle_username {
        if !oracle_username.is_empty() {
            match user_repo.find_by_username(oracle_username).await {
                Ok(oracle_user) => Some(oracle_user.id),
                Err(_) => {
                    let template = NewMarketTemplate {
                        error: Some(format!("Oracle user '{}' not found", oracle_username)),
                        username,
                    };
                    return Err(Html(template.render().unwrap()));
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    match market_repo
        .create(&form.question, description, creator_id, oracle_id, end_date, 100.0)
        .await
    {
        Ok(market) => Ok(Redirect::to(&format!("/markets/{}", market.id))),
        Err(e) => {
            let template = NewMarketTemplate {
                error: Some(format!("Error creating market: {}", e)),
                username,
            };
            Err(Html(template.render().unwrap()))
        }
    }
}

pub async fn view_market(
    auth: OptionalAuth,
    State(db): State<Database>,
    Path(id): Path<i64>,
) -> Result<Html<String>, String> {
    let market_repo = MarketRepository::new(db.pool().clone());

    let username = if let Some(user_id) = auth.user_id {
        let user_repo = UserRepository::new(db.pool().clone());
        user_repo.find_by_id(user_id)
            .await
            .ok()
            .map(|u| u.username)
    } else {
        None
    };

    let market = market_repo
        .find_by_id(id)
        .await
        .map_err(|_| "Market not found".to_string())?;

    let yes_prob = LmsrPricing::implied_probability(market.q_yes, market.q_no, market.liquidity_param);
    let market_display = MarketDisplay {
        id: market.id,
        question: market.question.clone(),
        description: market.description.clone(),
        end_date: market.end_date.format("%Y-%m-%d %H:%M").to_string(),
        yes_probability: yes_prob * 100.0,
        no_probability: (1.0 - yes_prob) * 100.0,
        total_liquidity: market.total_liquidity(),
        resolved: market.resolved,
        outcome: market.outcome,
    };

    let can_resolve = if let Some(user_id) = auth.user_id {
        market.can_resolve_by(user_id)
    } else {
        false
    };

    // Fetch user positions for this market
    let user_positions = if let Some(user_id) = auth.user_id {
        let position_repo = PositionRepository::new(db.pool().clone());
        let all_positions = position_repo.find_by_user(user_id).await.unwrap_or_default();
        all_positions.into_iter()
            .filter(|p| p.market_id == id && p.shares > 0.0)
            .map(|p| UserPosition {
                side: p.side.to_string(),
                shares: p.shares,
                avg_price: p.avg_price,
            })
            .collect()
    } else {
        Vec::new()
    };

    let template = MarketDetailTemplate {
        market: market_display,
        can_resolve,
        username,
        user_positions,
    };

    Ok(Html(template.render().unwrap()))
}

pub async fn resolve_market(
    auth: RequireAuth,
    State(db): State<Database>,
    Path(id): Path<i64>,
    Form(form): Form<ResolveMarketForm>,
) -> Result<Redirect, String> {
    let outcome = match form.outcome.as_str() {
        "yes" => true,
        "no" => false,
        _ => return Err("Invalid outcome".to_string()),
    };

    let market_repo = MarketRepository::new(db.pool().clone());

    // Check if user is authorized to resolve
    let market = market_repo
        .find_by_id(id)
        .await
        .map_err(|_| "Market not found".to_string())?;

    if !market.can_resolve_by(auth.user_id) {
        return Err("Only the designated oracle can resolve this market".to_string());
    }

    // Resolve the market
    market_repo
        .resolve(id, outcome)
        .await
        .map_err(|e| format!("Error resolving market: {}", e))?;

    // Process payouts
    process_payouts(&db, id, outcome)
        .await
        .map_err(|e| format!("Error processing payouts: {}", e))?;

    Ok(Redirect::to(&format!("/markets/{}", id)))
}

/// Process payouts for a resolved market
/// Winners receive $1 per share, losers receive $0
async fn process_payouts(db: &Database, market_id: i64, outcome: bool) -> Result<(), String> {
    use crate::repository::PositionRepository;

    let position_repo = PositionRepository::new(db.pool().clone());
    let user_repo = UserRepository::new(db.pool().clone());

    // Get all positions for this market
    let positions = position_repo
        .find_by_market(market_id)
        .await
        .map_err(|e| format!("Error fetching positions: {}", e))?;

    let winning_side = if outcome {
        MarketSide::Yes
    } else {
        MarketSide::No
    };

    // Pay out winners
    for position in positions {
        if position.side == winning_side && position.shares > 0.0 {
            // Each winning share pays out $1
            let payout = position.payout_if_wins();

            user_repo
                .add_balance(position.user_id, payout)
                .await
                .map_err(|e| format!("Error adding payout to user {}: {}", position.user_id, e))?;
        }
        // Losers get nothing (their shares are worthless)
    }

    Ok(())
}

