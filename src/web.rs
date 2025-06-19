use axum::Router;
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod common;
use common::fetcher::WebFetcher;

const BIND_ADDRESS: &str = "127.0.0.1:8040";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get port number from command line arguments
    let args: Vec<String> = env::args().collect();
    let bind_address = if args.len() > 1 {
        format!("127.0.0.1:{}", args[1])
    } else {
        BIND_ADDRESS.to_string()
    };

    let service = StreamableHttpService::new(
        || Ok(WebFetcher::new()),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    let router = Router::new().nest_service("/mcp", service);
    let tcp_listener = match tokio::net::TcpListener::bind(&bind_address).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::warn!(
                "Failed to bind to {}: {}. Trying random port...",
                bind_address,
                e
            );
            tokio::net::TcpListener::bind("127.0.0.1:0").await?
        }
    };
    let addr = tcp_listener.local_addr()?;
    tracing::info!("Listening on http://{}", addr);

    // Future for shutdown after receiving Ctrl+C or other signal, then wait 10 seconds before shutdown
    let shutdown = async {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("Shutdown signal received");
        std::process::exit(0);
    };

    let _ = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(shutdown)
        .await;
    Ok(())
}
