#[derive(Debug)]
pub enum CustomErrorKind {
    InsufficientFunds(String),
    InvalidAmount(String),
}

impl std::fmt::Display for CustomErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomErrorKind::InsufficientFunds(msg) => write!(f, "InsufficientFunds: {}", msg),
            CustomErrorKind::InvalidAmount(msg) => write!(f, "InvalidAmount: {}", msg),
        }
    }
}

impl std::error::Error for CustomErrorKind {}

pub async fn process_payment(
    customer_id: u32,
    amount: f64,
    wud: wud::WudClient,
) -> Result<(), CustomErrorKind> {

    if amount > 50.0 {
        wud.report(CustomErrorKind::InvalidAmount("Something went really wrong in payment".into())).await;
    }

    Ok(())
}
