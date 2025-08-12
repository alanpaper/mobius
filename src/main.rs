use crate::cli::alter::Alter;
use std::{error::Error};
mod cli;
mod markdown;
mod models;
mod session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Alter::new();
    app.run().await?;
    Ok(())
}
