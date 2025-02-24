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

    let dir = args
        .get(1)
        .ok_or_else(|| SetupError::new("Need to supply dir"))?
        .to_owned();

    setup(dir).await.map_err(SetupError::new)?;
    Ok(())
}
