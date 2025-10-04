use crate::domain::{Position, PositionId, UserId, MarketId, MarketSide};
use crate::repository::{Result, RepositoryError};
use sqlx::SqlitePool;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct PositionRepository {
    pool: SqlitePool,
}

impl PositionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_or_create(
        &self,
        user_id: UserId,
        market_id: MarketId,
        side: MarketSide,
    ) -> Result<Position> {
        // Try to find existing position
        if let Ok(position) = self.find_by_user_market_side(user_id, market_id, side).await {
            return Ok(position);
        }

        // Create new position
        let side_str = side.to_string();
        let result = sqlx::query!(
            r#"
            INSERT INTO positions (user_id, market_id, side, shares, avg_price)
            VALUES (?, ?, ?, 0.0, 0.0)
            RETURNING id, user_id, market_id, side, shares, avg_price, created_at, updated_at
            "#,
            user_id,
            market_id,
            side_str
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Position {
            id: result.id,
            user_id: result.user_id,
            market_id: result.market_id,
            side: result.side.parse().map_err(|_| {
                RepositoryError::Database(sqlx::Error::Decode(
                    "Invalid market side".into(),
                ))
            })?,
            shares: result.shares,
            avg_price: result.avg_price,
            created_at: DateTime::parse_from_rfc3339(&result.created_at)
                .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&result.updated_at)
                .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
        })
    }

    pub async fn find_by_user_market_side(
        &self,
        user_id: UserId,
        market_id: MarketId,
        side: MarketSide,
    ) -> Result<Position> {
        let side_str = side.to_string();
        let result = sqlx::query!(
            r#"
            SELECT id, user_id, market_id, side, shares, avg_price, created_at, updated_at
            FROM positions
            WHERE user_id = ? AND market_id = ? AND side = ?
            "#,
            user_id,
            market_id,
            side_str
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(Position {
            id: result.id.unwrap_or_default(),
            user_id: result.user_id,
            market_id: result.market_id,
            side: result.side.parse().map_err(|_| {
                RepositoryError::Database(sqlx::Error::Decode(
                    "Invalid market side".into(),
                ))
            })?,
            shares: result.shares,
            avg_price: result.avg_price,
            created_at: DateTime::parse_from_rfc3339(&result.created_at)
                .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&result.updated_at)
                .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
        })
    }

    pub async fn find_by_user(&self, user_id: UserId) -> Result<Vec<Position>> {
        let results = sqlx::query!(
            r#"
            SELECT id, user_id, market_id, side, shares, avg_price, created_at, updated_at
            FROM positions
            WHERE user_id = ? AND shares > 0
            ORDER BY updated_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        results
            .into_iter()
            .map(|r| {
                Ok(Position {
                    id: r.id.unwrap_or_default(),
                    user_id: r.user_id,
                    market_id: r.market_id,
                    side: r.side.parse().map_err(|_| {
                        RepositoryError::Database(sqlx::Error::Decode(
                            "Invalid market side".into(),
                        ))
                    })?,
                    shares: r.shares,
                    avg_price: r.avg_price,
                    created_at: DateTime::parse_from_rfc3339(&r.created_at)
                        .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&r.updated_at)
                        .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                        .with_timezone(&Utc),
                })
            })
            .collect()
    }

    pub async fn find_by_market(&self, market_id: MarketId) -> Result<Vec<Position>> {
        let results = sqlx::query!(
            r#"
            SELECT id, user_id, market_id, side, shares, avg_price, created_at, updated_at
            FROM positions
            WHERE market_id = ? AND shares > 0
            "#,
            market_id
        )
        .fetch_all(&self.pool)
        .await?;

        results
            .into_iter()
            .map(|r| {
                Ok(Position {
                    id: r.id.unwrap_or_default(),
                    user_id: r.user_id,
                    market_id: r.market_id,
                    side: r.side.parse().map_err(|_| {
                        RepositoryError::Database(sqlx::Error::Decode(
                            "Invalid market side".into(),
                        ))
                    })?,
                    shares: r.shares,
                    avg_price: r.avg_price,
                    created_at: DateTime::parse_from_rfc3339(&r.created_at)
                        .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&r.updated_at)
                        .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                        .with_timezone(&Utc),
                })
            })
            .collect()
    }

    pub async fn update(
        &self,
        id: PositionId,
        shares: f64,
        avg_price: f64,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query!(
            r#"
            UPDATE positions
            SET shares = ?, avg_price = ?, updated_at = ?
            WHERE id = ?
            "#,
            shares,
            avg_price,
            now,
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
