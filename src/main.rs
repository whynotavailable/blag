use blag::{
    models::{SetupError, SetupResult},
    setup,
};

#[tokio::main]
async fn main() -> SetupResult {
    setup().await.map_err(SetupError::new)?;
    Ok(())
}
