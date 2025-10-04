use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::UserId;

pub type MarketId = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketSide {
    Yes,
    No,
}

impl MarketSide {
    pub fn opposite(&self) -> Self {
        match self {
            MarketSide::Yes => MarketSide::No,
            MarketSide::No => MarketSide::Yes,
        }
    }
}

impl std::fmt::Display for MarketSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketSide::Yes => write!(f, "yes"),
            MarketSide::No => write!(f, "no"),
        }
    }
}

impl std::str::FromStr for MarketSide {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "yes" => Ok(MarketSide::Yes),
            "no" => Ok(MarketSide::No),
            _ => Err(format!("Invalid market side: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketStatus {
    Active,
    Closed,
    Resolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub id: MarketId,
    pub question: String,
    pub description: Option<String>,
    pub creator_id: UserId,
    pub oracle_id: Option<UserId>,
    pub end_date: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub resolved: bool,
    pub outcome: Option<bool>,
    // Legacy CPMM fields (kept for backward compatibility)
    pub yes_pool: f64,
    pub no_pool: f64,
    // LMSR fields
    pub q_yes: f64,
    pub q_no: f64,
    pub liquidity_param: f64,
    pub created_at: DateTime<Utc>,
}

impl Market {
    pub fn new(
        id: MarketId,
        question: String,
        description: Option<String>,
        creator_id: UserId,
        oracle_id: Option<UserId>,
        end_date: DateTime<Utc>,
        yes_pool: f64,
        no_pool: f64,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            question,
            description,
            creator_id,
            oracle_id,
            end_date,
            closed_at: None,
            resolved: false,
            outcome: None,
            yes_pool,
            no_pool,
            q_yes: 0.0,
            q_no: 0.0,
            liquidity_param: 100.0,
            created_at,
        }
    }

    pub fn new_lmsr(
        id: MarketId,
        question: String,
        description: Option<String>,
        creator_id: UserId,
        oracle_id: Option<UserId>,
        end_date: DateTime<Utc>,
        liquidity_param: f64,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            question,
            description,
            creator_id,
            oracle_id,
            end_date,
            closed_at: None,
            resolved: false,
            outcome: None,
            yes_pool: 0.0,  // Legacy field, not used
            no_pool: 0.0,   // Legacy field, not used
            q_yes: 0.0,
            q_no: 0.0,
            liquidity_param,
            created_at,
        }
    }

    pub fn get_oracle(&self) -> UserId {
        self.oracle_id.unwrap_or(self.creator_id)
    }

    pub fn can_resolve_by(&self, user_id: UserId) -> bool {
        !self.resolved && Utc::now() > self.end_date && self.get_oracle() == user_id
    }

    pub fn is_closed(&self) -> bool {
        self.closed_at.is_some() || Utc::now() > self.end_date
    }

    pub fn status(&self) -> MarketStatus {
        if self.resolved {
            MarketStatus::Resolved
        } else if self.is_closed() {
            MarketStatus::Closed
        } else {
            MarketStatus::Active
        }
    }

    pub fn can_trade(&self) -> bool {
        !self.is_closed() && !self.resolved
    }

    pub fn can_resolve(&self) -> bool {
        !self.resolved && self.is_closed()
    }

    pub fn resolve(&mut self, outcome: bool) -> Result<(), String> {
        if self.resolved {
            return Err("Market already resolved".to_string());
        }
        if !self.can_resolve() {
            return Err("Market cannot be resolved yet".to_string());
        }
        self.resolved = true;
        self.outcome = Some(outcome);
        Ok(())
    }

    pub fn total_liquidity(&self) -> f64 {
        // For LMSR markets, liquidity is represented by the liquidity parameter
        if self.liquidity_param > 0.0 {
            self.liquidity_param
        } else {
            // Fallback to legacy pool-based liquidity
            self.yes_pool + self.no_pool
        }
    }

    pub fn total_outstanding_shares(&self) -> f64 {
        self.q_yes + self.q_no
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_market_status() {
        let future = Utc::now() + Duration::days(1);
        let past = Utc::now() - Duration::days(1);

        let active = Market::new(1, "Q?".to_string(), None, 1, future, 100.0, 100.0, Utc::now());
        assert_eq!(active.status(), MarketStatus::Active);

        let closed = Market::new(1, "Q?".to_string(), None, 1, past, 100.0, 100.0, Utc::now());
        assert_eq!(closed.status(), MarketStatus::Closed);

        let mut resolved = Market::new(1, "Q?".to_string(), None, 1, past, 100.0, 100.0, Utc::now());
        resolved.resolve(true).unwrap();
        assert_eq!(resolved.status(), MarketStatus::Resolved);
    }

    #[test]
    fn test_can_trade() {
        let future = Utc::now() + Duration::days(1);
        let past = Utc::now() - Duration::days(1);

        let active = Market::new(1, "Q?".to_string(), None, 1, future, 100.0, 100.0, Utc::now());
        assert!(active.can_trade());

        let closed = Market::new(1, "Q?".to_string(), None, 1, past, 100.0, 100.0, Utc::now());
        assert!(!closed.can_trade());
    }

    #[test]
    fn test_resolve() {
        let past = Utc::now() - Duration::days(1);
        let mut market = Market::new(1, "Q?".to_string(), None, 1, past, 100.0, 100.0, Utc::now());

        assert!(market.resolve(true).is_ok());
        assert_eq!(market.outcome, Some(true));
        assert!(market.resolved);

        // Cannot resolve again
        assert!(market.resolve(false).is_err());
    }
}
