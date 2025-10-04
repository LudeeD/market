use crate::domain::{Market, MarketId, UserId};
use crate::repository::{Result, RepositoryError};
use sqlx::SqlitePool;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct MarketRepository {
    pool: SqlitePool,
}

impl MarketRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        question: &str,
        description: Option<&str>,
        creator_id: UserId,
        oracle_id: Option<UserId>,
        end_date: DateTime<Utc>,
        initial_liquidity: f64,
    ) -> Result<Market> {
        let end_date_str = end_date.to_rfc3339();
        let result = sqlx::query!(
            r#"
            INSERT INTO markets (question, description, creator_id, oracle_id, end_date, yes_pool, no_pool, q_yes, q_no, liquidity_param)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, question, description, creator_id, oracle_id, end_date, closed_at, resolved, outcome, yes_pool, no_pool, q_yes, q_no, liquidity_param, created_at
            "#,
            question,
            description,
            creator_id,
            oracle_id,
            end_date_str,
            0.0,  // Legacy yes_pool (not used)
            0.0,  // Legacy no_pool (not used)
            0.0,  // q_yes starts at 0
            0.0,  // q_no starts at 0
            initial_liquidity  // LMSR liquidity parameter
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Market {
            id: result.id,
            question: result.question,
            description: result.description,
            creator_id: result.creator_id,
            oracle_id: result.oracle_id,
            end_date: DateTime::parse_from_rfc3339(&result.end_date)
                .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
            closed_at: result.closed_at.as_ref().and_then(|s| DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
            resolved: result.resolved,
            outcome: result.outcome,
            yes_pool: result.yes_pool,
            no_pool: result.no_pool,
            q_yes: result.q_yes,
            q_no: result.q_no,
            liquidity_param: result.liquidity_param,
            created_at: DateTime::parse_from_rfc3339(&result.created_at)
                .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
        })
    }

    pub async fn find_by_id(&self, id: MarketId) -> Result<Market> {
        let result = sqlx::query!(
            r#"
            SELECT id, question, description, creator_id, oracle_id, end_date, closed_at, resolved, outcome, yes_pool, no_pool, q_yes, q_no, liquidity_param, created_at
            FROM markets
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(Market {
            id: result.id,
            question: result.question,
            description: result.description,
            creator_id: result.creator_id,
            oracle_id: result.oracle_id,
            end_date: DateTime::parse_from_rfc3339(&result.end_date)
                .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
            closed_at: result.closed_at.as_ref().and_then(|s| DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
            resolved: result.resolved,
            outcome: result.outcome,
            yes_pool: result.yes_pool,
            no_pool: result.no_pool,
            q_yes: result.q_yes,
            q_no: result.q_no,
            liquidity_param: result.liquidity_param,
            created_at: DateTime::parse_from_rfc3339(&result.created_at)
                .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
        })
    }

    pub async fn list_active(&self) -> Result<Vec<Market>> {
        let results = sqlx::query!(
            r#"
            SELECT id, question, description, creator_id, oracle_id, end_date, closed_at, resolved, outcome, yes_pool, no_pool, q_yes, q_no, liquidity_param, created_at
            FROM markets
            WHERE resolved = 0
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        results
            .into_iter()
            .map(|r| {
                Ok(Market {
                    id: r.id.unwrap_or_default(),
                    question: r.question,
                    description: r.description,
                    creator_id: r.creator_id,
                    oracle_id: r.oracle_id,
                    end_date: DateTime::parse_from_rfc3339(&r.end_date)
                        .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                        .with_timezone(&Utc),
                    closed_at: r.closed_at.as_ref().and_then(|s| DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
                    resolved: r.resolved,
                    outcome: r.outcome,
                    yes_pool: r.yes_pool,
                    no_pool: r.no_pool,
                    q_yes: r.q_yes,
                    q_no: r.q_no,
                    liquidity_param: r.liquidity_param,
                    created_at: DateTime::parse_from_rfc3339(&r.created_at)
                        .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                        .with_timezone(&Utc),
                })
            })
            .collect()
    }

    pub async fn list_all(&self) -> Result<Vec<Market>> {
        let results = sqlx::query!(
            r#"
            SELECT id, question, description, creator_id, oracle_id, end_date, closed_at, resolved, outcome, yes_pool, no_pool, q_yes, q_no, liquidity_param, created_at
            FROM markets
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        results
            .into_iter()
            .map(|r| {
                Ok(Market {
                    id: r.id,
                    question: r.question,
                    description: r.description,
                    creator_id: r.creator_id,
                    oracle_id: r.oracle_id,
                    end_date: DateTime::parse_from_rfc3339(&r.end_date)
                        .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                        .with_timezone(&Utc),
                    closed_at: r.closed_at.as_ref().and_then(|s| DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
                    resolved: r.resolved,
                    outcome: r.outcome,
                    yes_pool: r.yes_pool,
                    no_pool: r.no_pool,
                    q_yes: r.q_yes,
                    q_no: r.q_no,
                    liquidity_param: r.liquidity_param,
                    created_at: DateTime::parse_from_rfc3339(&r.created_at)
                        .map_err(|e| RepositoryError::Database(sqlx::Error::Decode(Box::new(e))))?
                        .with_timezone(&Utc),
                })
            })
            .collect()
    }

    pub async fn close(&self, id: MarketId) -> Result<()> {
        let closed_at = Utc::now().to_rfc3339();
        sqlx::query!(
            r#"
            UPDATE markets
            SET closed_at = ?
            WHERE id = ? AND closed_at IS NULL AND resolved = 0
            "#,
            closed_at,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_pools(&self, id: MarketId, yes_pool: f64, no_pool: f64) -> Result<()> {
        let result = sqlx::query!(
            r#"
            UPDATE markets
            SET yes_pool = ?, no_pool = ?
            WHERE id = ?
            "#,
            yes_pool,
            no_pool,
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    pub async fn update_outstanding_shares(&self, id: MarketId, q_yes: f64, q_no: f64) -> Result<()> {
        let result = sqlx::query!(
            r#"
            UPDATE markets
            SET q_yes = ?, q_no = ?
            WHERE id = ?
            "#,
            q_yes,
            q_no,
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    pub async fn resolve(&self, id: MarketId, outcome: bool) -> Result<()> {
        let outcome_int = if outcome { 1 } else { 0 };
        let result = sqlx::query!(
            r#"
            UPDATE markets
            SET resolved = 1, outcome = ?
            WHERE id = ? AND resolved = 0
            "#,
            outcome_int,
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::ConstraintViolation(
                "Market already resolved or not found".to_string(),
            ));
        }

        Ok(())
    }
}
