use crate::Database;
use crate::repository::{PriceSnapshotRepository, MarketRepository};
use crate::domain::{LmsrPricing, MarketSide};
use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceHistoryPoint {
    pub timestamp: String,
    pub yes_probability: f64,
    pub no_probability: f64,
}

#[derive(Debug, Serialize)]
pub struct PriceHistoryResponse {
    pub market_id: i64,
    pub data: Vec<PriceHistoryPoint>,
}

/// Get price history for a market
pub async fn get_price_history(
    State(db): State<Database>,
    Path(market_id): Path<i64>,
) -> Result<Json<PriceHistoryResponse>, StatusCode> {
    let snapshot_repo = PriceSnapshotRepository::new(db.pool().clone());

    let snapshots = snapshot_repo
        .get_history(market_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data: Vec<PriceHistoryPoint> = snapshots
        .into_iter()
        .map(|s| PriceHistoryPoint {
            timestamp: s.created_at.to_rfc3339(),
            yes_probability: s.yes_probability,
            no_probability: s.no_probability,
        })
        .collect();

    Ok(Json(PriceHistoryResponse { market_id, data }))
}

#[derive(Debug, Deserialize)]
pub struct CalculateCostQuery {
    pub shares: f64,
    pub side: String,
}

#[derive(Debug, Serialize)]
pub struct CostCalculationResponse {
    pub cost: f64,
    pub potential_payout: f64,
    pub potential_profit: f64,
    pub avg_price: f64,
}

/// Calculate the cost to buy shares
pub async fn calculate_buy_cost(
    State(db): State<Database>,
    Path(market_id): Path<i64>,
    Query(params): Query<CalculateCostQuery>,
) -> Result<Json<CostCalculationResponse>, StatusCode> {
    let market_repo = MarketRepository::new(db.pool().clone());

    let market = market_repo
        .find_by_id(market_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let side: MarketSide = params.side.parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let cost = LmsrPricing::calculate_buy_cost(
        market.q_yes,
        market.q_no,
        params.shares,
        side,
        market.liquidity_param,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    let potential_payout = params.shares; // Each share pays $1 if you win
    let potential_profit = potential_payout - cost;
    let avg_price = cost / params.shares;

    Ok(Json(CostCalculationResponse {
        cost,
        potential_payout,
        potential_profit,
        avg_price,
    }))
}
