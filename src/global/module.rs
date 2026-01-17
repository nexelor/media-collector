use tokio::sync::mpsc;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use std::num::NonZeroU32;
use governor::{Quota, RateLimiter as GovernorRateLimiter};
use governor::clock::{Clock, DefaultClock};
use governor::state::{InMemoryState, NotKeyed};

use super::{database::DatabaseInstance, error::AppError};

/// Message types that can be sent to parent modules
#[derive(Debug, Clone)]
pub enum ModuleMessage {
    Shutdown,
    Custom(String),
}

/// Response from child module operations
#[derive(Debug)]
pub struct ModuleResponse<T> {
    pub data: T,
}

/// Trait for parent modules that run continuously
pub trait ParentModule: Send + Sync {
    /// Module name for logging
    fn name(&self) -> &str;
    
    /// Initialize and run the module
    fn run(
        &self,
        db: Arc<DatabaseInstance>,
        rx: mpsc::Receiver<ModuleMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + '_>>;
}

/// Trait for child modules that are spawned on demand
pub trait ChildModule: Send + Sync {
    type Input: Send;
    type Output: Send;
    
    /// Module name for logging
    fn name(&self) -> &str;
    
    /// Execute the child module's task
    fn execute(
        &self,
        db: Arc<DatabaseInstance>,
        client: reqwest::Client,
        input: Self::Input,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Output, AppError>> + Send + '_>>;
}

/// Rate limiter for API requests using the governor crate
pub struct RateLimiter {
    limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    name: String,
    requests_per_second: f64,
}

impl RateLimiter {
    /// Create a new rate limiter using governor
    /// `requests_per_second` - maximum number of requests allowed per second
    pub fn new(name: &str, requests_per_second: f64) -> Self {
        // Convert requests per second to a quota
        // For sub-second rates, we use milliseconds precision
        let quota = if requests_per_second >= 1.0 {
            Quota::per_second(NonZeroU32::new(requests_per_second as u32).unwrap())
        } else {
            // For rates < 1 req/s, calculate the interval
            let interval_ms = (1000.0 / requests_per_second) as u64;
            Quota::with_period(std::time::Duration::from_millis(interval_ms))
                .unwrap()
        };

        Self {
            limiter: Arc::new(GovernorRateLimiter::direct(quota)),
            name: name.to_string(),
            requests_per_second,
        }
    }

    /// Acquire permission to make a request
    /// This will wait asynchronously if the rate limit is exceeded
    pub async fn acquire(&self) {
        loop {
            match self.limiter.check() {
                Ok(_) => {
                    println!("[{}] Rate limiter: permit acquired ({:.1} req/s)", 
                        self.name, self.requests_per_second);
                    break;
                }
                Err(not_until) => {
                    let clock = DefaultClock::default();
                    let wait_duration = not_until.wait_time_from(clock.now());
                    tokio::time::sleep(wait_duration).await;
                }
            }
        }
    }

    /// Try to acquire permission without waiting
    /// Returns Ok(()) if successful, Err with wait duration if rate limited
    pub fn try_acquire(&self) -> Result<(), std::time::Duration> {
        match self.limiter.check() {
            Ok(_) => Ok(()),
            Err(not_until) => {
                let clock = DefaultClock::default();
                let wait_duration = not_until.wait_time_from(clock.now());
                Err(wait_duration)
            }
        }
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            limiter: self.limiter.clone(),
            name: self.name.clone(),
            requests_per_second: self.requests_per_second,
        }
    }
}

/// Handle for controlling a parent module
pub struct ModuleHandle {
    pub name: String,
    pub tx: mpsc::Sender<ModuleMessage>,
}

impl ModuleHandle {
    pub async fn shutdown(&self) -> Result<(), AppError> {
        self.tx.send(ModuleMessage::Shutdown).await
            .map_err(|e| AppError::Module(format!("Failed to send shutdown: {}", e)))
    }
}