use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type UserId = i64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub password_hash: String,
    pub balance: f64,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(id: UserId, username: String, password_hash: String, balance: f64, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            username,
            password_hash,
            balance,
            created_at,
        }
    }

    pub fn can_afford(&self, amount: f64) -> bool {
        self.balance >= amount
    }

    pub fn deduct_balance(&mut self, amount: f64) -> Result<(), String> {
        if !self.can_afford(amount) {
            return Err("Insufficient balance".to_string());
        }
        self.balance -= amount;
        Ok(())
    }

    pub fn add_balance(&mut self, amount: f64) {
        self.balance += amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_afford() {
        let user = User::new(1, "test".to_string(), "hash".to_string(), 100.0, Utc::now());
        assert!(user.can_afford(50.0));
        assert!(user.can_afford(100.0));
        assert!(!user.can_afford(100.1));
    }

    #[test]
    fn test_deduct_balance() {
        let mut user = User::new(1, "test".to_string(), "hash".to_string(), 100.0, Utc::now());
        assert!(user.deduct_balance(50.0).is_ok());
        assert_eq!(user.balance, 50.0);
        assert!(user.deduct_balance(60.0).is_err());
        assert_eq!(user.balance, 50.0); // Should not change on error
    }

    #[test]
    fn test_add_balance() {
        let mut user = User::new(1, "test".to_string(), "hash".to_string(), 100.0, Utc::now());
        user.add_balance(50.0);
        assert_eq!(user.balance, 150.0);
    }
}
