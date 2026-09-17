//! Lekhani IBus Daemon Main Entrypoint

#[cfg(unix)]
mod engine;

#[cfg(unix)]
use engine::LekhaniIBusEngine;
#[cfg(unix)]
use tracing::info;
#[cfg(unix)]
use zbus::connection::Builder;

#[cfg(unix)]
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

#[cfg(not(unix))]
fn main() {
    eprintln!("Lekhani IBus engine daemon is only supported on Linux/Unix platforms.");
}
