use blag::setup;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use whynot_errors::{SetupError, SetupResult};

#[tokio::main]
async fn main() -> SetupResult {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    setup().await.map_err(SetupError::new)?;
    Ok(())
}
