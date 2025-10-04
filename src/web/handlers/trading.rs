use crate::Database;
use crate::repository::{MarketRepository, PositionRepository, UserRepository, PriceSnapshotRepository};
use crate::domain::{LmsrPricing, MarketSide};
use crate::web::filters;
use crate::web::session::RequireAuth;
use axum::{
    extract::{State, Path},
    response::{Html, Redirect},
    Form,
};
use askama::Template;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "positions.html")]
struct PositionsTemplate {
    positions: Vec<PositionDisplay>,
    balance: f64,
    username: Option<String>,
}

struct PositionDisplay {
    market_id: i64,
    market_question: String,
    side: String,
    shares: f64,
    avg_price: f64,
    total_cost: f64,
    payout_if_win: f64,
    profit_if_win: f64,
    loss_if_lose: f64,
    market_resolved: bool,
    won: bool,
}

#[derive(Deserialize)]
pub struct TradeForm {
    shares: f64,
    side: String,
}

pub async fn buy_shares(
    auth: RequireAuth,
    State(db): State<Database>,
    Path(market_id): Path<i64>,
    Form(form): Form<TradeForm>,
) -> Result<Redirect, String> {
    if form.shares <= 0.0 {
        return Err("Shares must be positive".to_string());
    }

    let side: MarketSide = form.side.parse()
        .map_err(|e| format!("Invalid side: {}", e))?;

    let market_repo = MarketRepository::new(db.pool().clone());
    let user_repo = UserRepository::new(db.pool().clone());
    let position_repo = PositionRepository::new(db.pool().clone());

    // Get market
    let market = market_repo
        .find_by_id(market_id)
        .await
        .map_err(|_| "Market not found".to_string())?;

    if !market.can_trade() {
        return Err("Market is not open for trading".to_string());
    }

    // Calculate cost using LMSR
    let cost = LmsrPricing::calculate_buy_cost(
        market.q_yes,
        market.q_no,
        form.shares,
        side,
        market.liquidity_param
    ).map_err(|e| format!("Error calculating cost: {}", e))?;

    let user_id = auth.user_id;

    // Check user balance
    let user = user_repo
        .find_by_id(user_id)
        .await
        .map_err(|_| "User not found".to_string())?;

    if !user.can_afford(cost) {
        return Err("Insufficient balance".to_string());
    }

    // Deduct from user balance
    user_repo
        .deduct_balance(user_id, cost)
        .await
        .map_err(|e| format!("Error deducting balance: {}", e))?;

    // Update market outstanding shares (LMSR)
    let (new_q_yes, new_q_no) = match side {
        MarketSide::Yes => (market.q_yes + form.shares, market.q_no),
        MarketSide::No => (market.q_yes, market.q_no + form.shares),
    };

    market_repo
        .update_outstanding_shares(market_id, new_q_yes, new_q_no)
        .await
        .map_err(|e| format!("Error updating outstanding shares: {}", e))?;

    // Record price snapshot
    let new_probability = LmsrPricing::implied_probability(new_q_yes, new_q_no, market.liquidity_param);
    let snapshot_repo = PriceSnapshotRepository::new(db.pool().clone());
    let _ = snapshot_repo
        .create(market_id, new_probability, 1.0 - new_probability, new_q_yes, new_q_no)
        .await;

    // Update user position
    let mut position = position_repo
        .find_or_create(user_id, market_id, side)
        .await
        .map_err(|e| format!("Error getting position: {}", e))?;

    let price_per_share = cost / form.shares;
    position.add_shares(form.shares, price_per_share);

    position_repo
        .update(position.id, position.shares, position.avg_price)
        .await
        .map_err(|e| format!("Error updating position: {}", e))?;

    Ok(Redirect::to(&format!("/markets/{}", market_id)))
}

pub async fn sell_shares(
    auth: RequireAuth,
    State(db): State<Database>,
    Path(market_id): Path<i64>,
    Form(form): Form<TradeForm>,
) -> Result<Redirect, String> {
    if form.shares <= 0.0 {
        return Err("Shares must be positive".to_string());
    }

    let side: MarketSide = form.side.parse()
        .map_err(|e| format!("Invalid side: {}", e))?;

    let market_repo = MarketRepository::new(db.pool().clone());
    let user_repo = UserRepository::new(db.pool().clone());
    let position_repo = PositionRepository::new(db.pool().clone());

    // Get market
    let market = market_repo
        .find_by_id(market_id)
        .await
        .map_err(|_| "Market not found".to_string())?;

    if !market.can_trade() {
        return Err("Market is not open for trading".to_string());
    }

    let user_id = auth.user_id;

    // Check user position
    let mut position = position_repo
        .find_by_user_market_side(user_id, market_id, side)
        .await
        .map_err(|_| "Position not found".to_string())?;

    if position.shares < form.shares {
        return Err("Insufficient shares to sell".to_string());
    }

    // Calculate proceeds using LMSR
    let proceeds = LmsrPricing::calculate_sell_proceeds(
        market.q_yes,
        market.q_no,
        form.shares,
        side,
        market.liquidity_param
    ).map_err(|e| format!("Error calculating proceeds: {}", e))?;

    // Add to user balance
    user_repo
        .add_balance(user_id, proceeds)
        .await
        .map_err(|e| format!("Error adding balance: {}", e))?;

    // Update market outstanding shares (LMSR)
    let (new_q_yes, new_q_no) = match side {
        MarketSide::Yes => (market.q_yes - form.shares, market.q_no),
        MarketSide::No => (market.q_yes, market.q_no - form.shares),
    };

    market_repo
        .update_outstanding_shares(market_id, new_q_yes, new_q_no)
        .await
        .map_err(|e| format!("Error updating outstanding shares: {}", e))?;

    // Record price snapshot
    let new_probability = LmsrPricing::implied_probability(new_q_yes, new_q_no, market.liquidity_param);
    let snapshot_repo = PriceSnapshotRepository::new(db.pool().clone());
    let _ = snapshot_repo
        .create(market_id, new_probability, 1.0 - new_probability, new_q_yes, new_q_no)
        .await;

    // Update user position
    position.remove_shares(form.shares)
        .map_err(|e| format!("Error removing shares: {}", e))?;

    position_repo
        .update(position.id, position.shares, position.avg_price)
        .await
        .map_err(|e| format!("Error updating position: {}", e))?;

    Ok(Redirect::to(&format!("/markets/{}", market_id)))
}

pub async fn view_positions(auth: RequireAuth, State(db): State<Database>) -> Html<String> {
    let user_id = auth.user_id;

    let position_repo = PositionRepository::new(db.pool().clone());
    let market_repo = MarketRepository::new(db.pool().clone());
    let user_repo = UserRepository::new(db.pool().clone());

    let positions = position_repo.find_by_user(user_id).await.unwrap_or_default();
    let user = user_repo.find_by_id(user_id).await.unwrap();

    let mut positions_display = Vec::new();
    for position in positions {
        if let Ok(market) = market_repo.find_by_id(position.market_id).await {
            let total_cost = position.shares * position.avg_price;
            let payout_if_win = position.shares; // $1 per share
            let profit_if_win = payout_if_win - total_cost;
            let loss_if_lose = total_cost; // You lose what you paid

            // Determine if position won (if market is resolved)
            let won = if market.resolved {
                if let Some(outcome) = market.outcome {
                    (outcome && position.side == MarketSide::Yes) ||
                    (!outcome && position.side == MarketSide::No)
                } else {
                    false
                }
            } else {
                false
            };

            positions_display.push(PositionDisplay {
                market_id: market.id,
                market_question: market.question,
                side: position.side.to_string(),
                shares: position.shares,
                avg_price: position.avg_price,
                total_cost,
                payout_if_win,
                profit_if_win,
                loss_if_lose,
                market_resolved: market.resolved,
                won,
            });
        }
    }

    let template = PositionsTemplate {
        positions: positions_display,
        balance: user.balance,
        username: Some(user.username),
    };
    Html(template.render().unwrap())
}
