mod user_repo;
mod market_repo;
mod position_repo;
mod price_snapshot_repo;

pub use user_repo::UserRepository;
pub use market_repo::MarketRepository;
pub use position_repo::PositionRepository;
pub use price_snapshot_repo::PriceSnapshotRepository;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

pub type Result<T> = std::result::Result<T, RepositoryError>;
