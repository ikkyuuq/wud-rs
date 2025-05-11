use backtrace::{Backtrace, BacktraceFrame};
use log::{debug, error, info, warn};
use thiserror::Error;

pub struct Config {
    pub app_name: String,
    pub slack_webhook_url: String,
}

pub struct WudClient {
    pub config: Config,
    pub client: reqwest::Client,
}

struct WudError {
    source: Box<dyn std::error::Error + Send + Sync>,
    error_type: String,
    backtrace: Option<Backtrace>,
}

struct ErrorEvent {
    app_name: String,
    error_type: String,
    error_message: String,
    backtrace_frames: Vec<BacktraceFrame>,
}

impl std::fmt::Display for WudError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl std::fmt::Debug for WudError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(backtrace) = &self.backtrace {
            write!(f, "{}\n\nBacktrace: {:?}", self.source, backtrace)
        } else {
            write!(f, "{}", self.source)
        }
    }
}

impl std::error::Error for WudError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.source)
    }
}

impl WudError {
    fn new<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            source: Box::new(error),
            error_type: std::any::type_name::<E>().to_string(),
            backtrace: Some(Backtrace::new()),
        }
    }
}

impl WudClient {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    pub async fn report<E>(self, error: E) -> Result<(), Box<dyn std::error::Error>>
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        let error = WudError::new(error);

        let error_event = self.create_error_event(error);

        error!("{}:{}", error_event.error_type, error_event.error_message);

        self.send_to_slack(error_event).await?;

        Ok(())
    }

    async fn send_to_slack(
        &self,
        error_event: ErrorEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = self.config.slack_webhook_url.clone();

        let client = reqwest::Client::new();

        let mut backtrace_text = String::new();
        for frame in error_event.backtrace_frames.iter() {
            for symbol in frame.symbols() {
                if let (Some(filename), Some(lineno), Some(name)) = (
                    symbol.filename(),
                    symbol.lineno(),
                    symbol.name().map(|s| format!("{:#}", s)),
                ) {
                    if filename.to_string_lossy().contains("/home")
                        && !filename.to_string_lossy().contains(".cargo")
                        && !filename.to_string_lossy().contains(".rustc")
                        && !filename.to_string_lossy().contains("lib.rs")
                    {
                        let file = filename.display().to_string();
                        let line = lineno;
                        let func = name.clone();
                        if !backtrace_text.is_empty() {
                            backtrace_text.push_str("\n");
                        }
                        backtrace_text.push_str(&format!("*{}* in `{}` at `{}`", file, func, line));
                        debug!(
                            "Reporting error to Slack: {}",
                            format!("\n{} in {} at {}", file, func, line)
                        );
                    }
                }
            }
        }

        let mut payload = serde_json::json!({
            "attachments": [
                {
                    "color": "#FF0000",
                    "blocks": [
                        {
                            "type": "header",
                            "text": {
                                "type": "plain_text",
                                "text": format!(":warning: WUD Report | {}", error_event.app_name),
                                "emoji": true,
                            },
                        },
                        {
                            "type": "section",
                            "fields": [
                                {
                                    "type": "mrkdwn",
                                    "text": format!(
                                        "*Error Type:*\n{}",
                                        error_event.error_type
                                    ),
                                },
                                {
                                    "type": "mrkdwn",
                                    "text": format!(
                                        "*Error Message:*\n{}",
                                        error_event.error_message
                                    ),
                                },
                            ],
                        },
                    ]
                }
            ],
        });

        if !backtrace_text.is_empty() {
            payload["attachments"][0]["blocks"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": format!("*Backtrace:*\n```{}```", backtrace_text),
                    }
                }));
        }

        let resp = client.post(&url).json(&payload).send().await?;
        debug!("Response: {}", resp.text().await?);

        Ok(())
    }

    fn create_error_event(&self, error: WudError) -> ErrorEvent {
        let backtrace_frames = self.backtrace_to_backtrace_frames(&error);

        ErrorEvent {
            app_name: self.config.app_name.clone(),
            error_type: error.error_type.clone(),
            error_message: error.source.to_string(),
            backtrace_frames,
        }
    }

    fn backtrace_to_backtrace_frames(&self, error: &WudError) -> Vec<BacktraceFrame> {
        let mut bt = Vec::new();
        if let Some(backtrace) = &error.backtrace {
            for frame in backtrace.frames().iter().take(10) {
                for symbol in frame.symbols() {
                    if let (Some(filename), Some(lineno), Some(name)) = (
                        symbol.filename(),
                        symbol.lineno(),
                        symbol.name().map(|s| format!("{:#}", s)),
                    ) {
                        if filename.to_string_lossy().contains("/home")
                            && !filename.to_string_lossy().contains(".cargo")
                            && !filename.to_string_lossy().contains(".rustc")
                            && !filename.to_string_lossy().contains("lib.rs")
                        {
                            let file = filename.display().to_string();
                            let line = lineno;
                            let func = name.clone();

                            bt.push(frame.clone());
                        }
                    }
                }
            }
        }
        bt
    }
}
