use crate::domain::PriceSnapshot;
use chrono::Utc;
use sqlx::{FromRow, SqlitePool};

#[derive(FromRow)]
struct PriceSnapshotRow {
    id: i64,
    market_id: i64,
    yes_probability: f64,
    no_probability: f64,
    q_yes: f64,
    q_no: f64,
    created_at: String,
}

impl From<PriceSnapshotRow> for PriceSnapshot {
    fn from(row: PriceSnapshotRow) -> Self {
        PriceSnapshot {
            id: row.id,
            market_id: row.market_id,
            yes_probability: row.yes_probability,
            no_probability: row.no_probability,
            q_yes: row.q_yes,
            q_no: row.q_no,
            created_at: row.created_at.parse().unwrap_or_else(|_| Utc::now()),
        }
    }
}

pub struct PriceSnapshotRepository {
    pool: SqlitePool,
}

impl PriceSnapshotRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new price snapshot
    pub async fn create(
        &self,
        market_id: i64,
        yes_probability: f64,
        no_probability: f64,
        q_yes: f64,
        q_no: f64,
    ) -> Result<PriceSnapshot, sqlx::Error> {
        let created_at = Utc::now().to_rfc3339();

        let row = sqlx::query_as::<_, PriceSnapshotRow>(
            r#"
            INSERT INTO price_snapshots (market_id, yes_probability, no_probability, q_yes, q_no, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id, market_id, yes_probability, no_probability, q_yes, q_no, created_at
            "#,
        )
        .bind(market_id)
        .bind(yes_probability)
        .bind(no_probability)
        .bind(q_yes)
        .bind(q_no)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    /// Get price history for a market, ordered by time ascending
    pub async fn get_history(&self, market_id: i64) -> Result<Vec<PriceSnapshot>, sqlx::Error> {
        let rows = sqlx::query_as::<_, PriceSnapshotRow>(
            r#"
            SELECT id, market_id, yes_probability, no_probability, q_yes, q_no, created_at
            FROM price_snapshots
            WHERE market_id = ?
            ORDER BY created_at ASC
            "#,
        )
        .bind(market_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Get price history for a market with a limit
    pub async fn get_history_limit(
        &self,
        market_id: i64,
        limit: i64,
    ) -> Result<Vec<PriceSnapshot>, sqlx::Error> {
        let rows = sqlx::query_as::<_, PriceSnapshotRow>(
            r#"
            SELECT id, market_id, yes_probability, no_probability, q_yes, q_no, created_at
            FROM price_snapshots
            WHERE market_id = ?
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(market_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        // Reverse to get ascending order
        let mut snapshots: Vec<PriceSnapshot> = rows.into_iter().map(Into::into).collect();
        snapshots.reverse();
        Ok(snapshots)
    }

    /// Get latest snapshot for a market
    pub async fn get_latest(&self, market_id: i64) -> Result<Option<PriceSnapshot>, sqlx::Error> {
        let row = sqlx::query_as::<_, PriceSnapshotRow>(
            r#"
            SELECT id, market_id, yes_probability, no_probability, q_yes, q_no, created_at
            FROM price_snapshots
            WHERE market_id = ?
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(market_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }
}
