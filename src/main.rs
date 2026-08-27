use http::Method;
use noshi_back::router::create_router;
use noshi_back::shared::config::Config;
use noshi_back::shared::db::{establish_pool, run_migrations};
use noshi_back::state::AppState;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "noshi_back=debug,tower_http=debug,info".into()),
        )
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
        &config,
    )
    .expect("Failed to initialize AppState");
    let app = create_router(state);
    let origins = [
        "https://nezia.app".parse().unwrap(),
        "http://portal.nezia.app".parse().unwrap(),
    ];
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    let app = app.layer(cors).layer(
        TraceLayer::new_for_http()
            .make_span_with(|req: &axum::http::Request<_>| {
                let request_id = Uuid::new_v4().to_string();
                let method = req.method().clone();
                let uri = req.uri().clone();
                tracing::debug_span!(
                    "request",
                    method = %method,
                    uri = %uri,
                    request_id = %request_id,
                )
            })
            .on_request(|_req: &axum::http::Request<_>, _span: &tracing::Span| {
                tracing::debug!("incoming request");
            })
            .on_response(
                |response: &axum::http::Response<_>,
                 latency: std::time::Duration,
                 _span: &tracing::Span| {
                    tracing::debug!(
                        status = %response.status(),
                        latency_ms = latency.as_millis() as u64,
                        "response sent"
                    );
                },
            ),
    );

    let addr = SocketAddr::from((
        config.server_host.parse::<std::net::IpAddr>().unwrap(),
        config.server_port,
    ));
    info!("Server starting on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
