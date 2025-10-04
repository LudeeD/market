use market::Database;
use market::web::create_router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tower_sessions::Expiry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "market=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:market.db".to_string());

    tracing::info!("Connecting to database: {}", database_url);
    let db = Database::new(&database_url).await?;

    // Run migrations
    tracing::info!("Running database migrations");
    db.run_migrations().await?;

    // Create session store and layer
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_expiry(Expiry::OnInactivity(time::Duration::days(7)));

    // Create router with shared state and session layer
    let app = create_router()
        .layer(session_layer)
        .with_state(db);

    // Start server
    let addr = "127.0.0.1:3000";
    tracing::info!("Starting server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
