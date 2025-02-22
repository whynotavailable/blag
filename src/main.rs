use blag::setup;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use whynot_errors::{SetupError, SetupResult};

#[tokio::main]
async fn main() -> SetupResult {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|err| SetupError::new(format!("setting default subscriber failed: {}", err)))?;

    setup().await.map_err(SetupError::new)?;
    Ok(())
}
