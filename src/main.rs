use std::env;

use anyhow::Result;
use payment::process_payment;
use thiserror::Error;
use wud::{Config, WudClient};
use dotenv::dotenv;

mod payment;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    let slack_webhook_url = env::var("SLACK_WEBHOOK_URL")?;

    let wud_config = Config {
        app_name: "Test App".to_string(),
        slack_webhook_url,
    };

    let wud_client = WudClient::new(wud_config);

    process_payment(1, 100.0, wud_client).await?;

    Ok(())
}

