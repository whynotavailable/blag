use std::env;

use blag::setup;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use whynot_errors::{SetupError, SetupResult};

#[tokio::main]
async fn main() -> SetupResult {
    let args: Vec<String> = env::args().collect();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|err| SetupError::new(format!("setting default subscriber failed: {}", err)))?;

    setup(args[1].clone()).await.map_err(SetupError::new)?;
    Ok(())
}
