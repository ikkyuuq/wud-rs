use backtrace::Backtrace;
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
    backtrace: Option<Backtrace>,
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
        let error_type = std::any::type_name::<E>().to_string();
        error!("{error_type}:{error}");

        // TODO : create a new struct calls error_event to store the error_type and error_message, and backtrace
        // TODO : send the error event to the provider
        if let Some(backtrace) = error.backtrace {
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
                            error!("{file} in {func} at {line}",);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
