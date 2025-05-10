use anyhow::Result;
use payment::process_payment;
use thiserror::Error;
use wud::{Config, WudClient};

mod payment;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let wud_config = Config {
        app_name: "wud".to_string(),
        slack_webhook_url:
            "https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX"
                .to_string(),
    };

    let wud_client = WudClient::new(wud_config);

    process_payment(1, 100.0, wud_client).await?;

    Ok(())
}

