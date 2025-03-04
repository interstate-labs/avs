use std::{fs, net::SocketAddr, sync::Arc};

use dotenvy::dotenv;
use square_number_dss_aggregator::{aggregator::OperatorState, task::TaskService};
use tokio::net::TcpListener;
use tokio::signal;
use tower::ServiceBuilder;
use tower_governor::{governor::GovernorConfig, GovernorLayer};
use tower_http::trace::{self, TraceLayer};
use tracing::{warn, Level};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if fs::metadata(".env").is_ok() {
        dotenv().ok();
    } else {
        warn!("No .env file not found.");
    }
    let config = envy::from_env::<square_number_dss_aggregator::Config>()?;
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();

    let governor_config = Arc::new(GovernorConfig::default());
    let operator_state = Arc::new(OperatorState::new());
    let aggregator_app = square_number_dss_aggregator::routes(operator_state.clone());
    let app = aggregator_app
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_request(trace::DefaultOnRequest::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
                .on_failure(trace::DefaultOnFailure::new().level(Level::ERROR)),
        )
        .layer(ServiceBuilder::new().layer(GovernorLayer {
            config: governor_config.clone(),
        }));

    let listener = TcpListener::bind((config.host, config.port)).await?;

    // let task_service = Arc::new(TaskService::new(operator_state, config)?);
    let verifer_service= Arc::new(TaskService::new(operator_state, config)?);
    // tokio::spawn(async move { task_service.start().await });
    tokio::spawn(async move { verifer_service.start().await });

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
