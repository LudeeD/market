use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::{UserId, MarketId, MarketSide};

pub type PositionId = i64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: PositionId,
    pub user_id: UserId,
    pub market_id: MarketId,
    pub side: MarketSide,
    pub shares: f64,
    pub avg_price: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Position {
    pub fn new(
        id: PositionId,
        user_id: UserId,
        market_id: MarketId,
        side: MarketSide,
        shares: f64,
        avg_price: f64,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            market_id,
            side,
            shares,
            avg_price,
            created_at,
            updated_at,
        }
    }

    /// Add shares to this position, updating the average price
    pub fn add_shares(&mut self, new_shares: f64, price: f64) {
        if new_shares <= 0.0 {
            return;
        }

        let total_cost = (self.shares * self.avg_price) + (new_shares * price);
        self.shares += new_shares;
        self.avg_price = if self.shares > 0.0 {
            total_cost / self.shares
        } else {
            0.0
        };
        self.updated_at = Utc::now();
    }

    /// Remove shares from this position
    pub fn remove_shares(&mut self, shares_to_remove: f64) -> Result<(), String> {
        if shares_to_remove > self.shares {
            return Err("Cannot remove more shares than held".to_string());
        }
        self.shares -= shares_to_remove;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Calculate the current value of this position at a given price
    pub fn value_at_price(&self, current_price: f64) -> f64 {
        self.shares * current_price
    }

    /// Calculate profit/loss compared to average purchase price
    pub fn profit_loss(&self, current_price: f64) -> f64 {
        self.shares * (current_price - self.avg_price)
    }

    /// Calculate payout if the market resolves in favor of this position
    pub fn payout_if_wins(&self) -> f64 {
        self.shares
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_shares() {
        let mut position = Position::new(
            1, 1, 1, MarketSide::Yes,
            10.0, 0.5, Utc::now(), Utc::now()
        );

        // Add 10 shares at 0.6
        position.add_shares(10.0, 0.6);
        assert_eq!(position.shares, 20.0);
        assert_eq!(position.avg_price, 0.55); // (10*0.5 + 10*0.6) / 20
    }

    #[test]
    fn test_remove_shares() {
        let mut position = Position::new(
            1, 1, 1, MarketSide::Yes,
            10.0, 0.5, Utc::now(), Utc::now()
        );

        assert!(position.remove_shares(5.0).is_ok());
        assert_eq!(position.shares, 5.0);

        assert!(position.remove_shares(10.0).is_err());
    }

    #[test]
    fn test_profit_loss() {
        let position = Position::new(
            1, 1, 1, MarketSide::Yes,
            10.0, 0.5, Utc::now(), Utc::now()
        );

        // Current price is 0.7, bought at 0.5
        assert!((position.profit_loss(0.7) - 2.0).abs() < 0.01); // 10 * (0.7 - 0.5)
        assert!((position.profit_loss(0.3) - (-2.0)).abs() < 0.01); // 10 * (0.3 - 0.5)
    }
}
