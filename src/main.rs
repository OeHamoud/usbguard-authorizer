mod event;
mod prompt;
mod authorizer;
mod logger;

use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    info!("USBGuard Authorizer starting...");
    info!("Listening for USB events via `usbguard watch`");

    if let Err(e) = authorizer::run().await {
        error!("Fatal error in authorizer: {:#}", e);
        std::process::exit(1);
    }

    Ok(())
}