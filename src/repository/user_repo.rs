use crate::domain::{User, UserId};
use crate::repository::{Result, RepositoryError};
use sqlx::SqlitePool;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct UserRepository {
    pool: SqlitePool,
}

impl UserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, username: &str, password_hash: &str) -> Result<User> {
        let result = sqlx::query!(
            r#"
            INSERT INTO users (username, password_hash, balance)
            VALUES (?, ?, 1000.0)
            RETURNING id, username, password_hash, balance, created_at
            "#,
            username,
            password_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return RepositoryError::ConstraintViolation("Username already exists".to_string());
                }
            }
            RepositoryError::Database(e)
        })?;

        Ok(User {
            id: result.id,
            username: result.username,
            password_hash: result.password_hash,
            balance: result.balance,
            created_at: DateTime::parse_from_rfc3339(&result.created_at)
                .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
        })
    }

    pub async fn find_by_id(&self, id: UserId) -> Result<User> {
        let result = sqlx::query!(
            r#"
            SELECT id, username, password_hash, balance, created_at
            FROM users
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(User {
            id: result.id,
            username: result.username,
            password_hash: result.password_hash,
            balance: result.balance,
            created_at: DateTime::parse_from_rfc3339(&result.created_at)
                .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
        })
    }

    pub async fn find_by_username(&self, username: &str) -> Result<User> {
        let result = sqlx::query!(
            r#"
            SELECT id, username, password_hash, balance, created_at
            FROM users
            WHERE username = ?
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(User {
            id: result.id.unwrap_or_default(),
            username: result.username,
            password_hash: result.password_hash,
            balance: result.balance,
            created_at: DateTime::parse_from_rfc3339(&result.created_at)
                .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
        })
    }

    pub async fn update_balance(&self, id: UserId, new_balance: f64) -> Result<()> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET balance = ?
            WHERE id = ?
            "#,
            new_balance,
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    pub async fn deduct_balance(&self, id: UserId, amount: f64) -> Result<()> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET balance = balance - ?
            WHERE id = ? AND balance >= ?
            "#,
            amount,
            id,
            amount
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::ConstraintViolation("Insufficient balance".to_string()));
        }

        Ok(())
    }

    pub async fn add_balance(&self, id: UserId, amount: f64) -> Result<()> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET balance = balance + ?
            WHERE id = ?
            "#,
            amount,
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}
