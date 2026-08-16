use http::Method;
use noshi_back::router::create_router;
use noshi_back::shared::config::Config;
use noshi_back::shared::db::{establish_pool, run_migrations};
use noshi_back::state::AppState;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env().expect("Failed to load config");

    let pool = establish_pool(&config.database_url, config.database_pool_size);
    run_migrations(&pool);

    let frontend_url = config
        .frontend_url
        .clone()
        .unwrap_or_else(|| "http://localhost:5173".to_string());
    let state = AppState::new(
        pool,
        config.jwt_secret.clone(),
        config.jwt_expiry_minutes,
        frontend_url,
    );
    let app = create_router(state);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    let app = app.layer(cors);

    let addr = SocketAddr::from((
        config.server_host.parse::<std::net::IpAddr>().unwrap(),
        config.server_port,
    ));
    tracing::info!("Server starting on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
