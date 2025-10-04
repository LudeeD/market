use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSnapshot {
    pub id: i64,
    pub market_id: i64,
    pub yes_probability: f64,
    pub no_probability: f64,
    pub q_yes: f64,
    pub q_no: f64,
    pub created_at: DateTime<Utc>,
}

impl PriceSnapshot {
    pub fn new(
        id: i64,
        market_id: i64,
        yes_probability: f64,
        no_probability: f64,
        q_yes: f64,
        q_no: f64,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            market_id,
            yes_probability,
            no_probability,
            q_yes,
            q_no,
            created_at,
        }
    }
}
