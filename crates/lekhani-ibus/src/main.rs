//! Lekhani IBus Daemon Main Entrypoint

mod engine;

use engine::LekhaniIBusEngine;
use tracing::info;
use zbus::connection::Builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting Lekhani IBus Input Method Engine...");

    let engine = LekhaniIBusEngine::new();

    let _conn = Builder::session()?
        .name("org.freedesktop.IBus.Lekhani")?
        .serve_at("/org/freedesktop/IBus/Engine", engine)?
        .build()
        .await?;

    info!("Lekhani IBus engine registered on DBus session bus");

    // Run until termination
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Lekhani IBus engine...");

    Ok(())
}
