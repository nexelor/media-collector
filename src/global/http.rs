// src/global/http.rs
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;

use crate::global::module::RateLimiter;
use crate::global::error::HttpError;

/// Manages HTTP clients with rate limiting for different APIs
#[derive(Clone)]
pub struct HttpClientManager {
    clients: Arc<ClientPool>,
}

struct ClientPool {
    default: ClientWithLimiter,
    my_anime_list: ClientWithLimiter,
    // Add more API-specific clients here
}

#[derive(Clone)]
pub struct ClientWithLimiter {
    pub client: Client,
    pub limiter: RateLimiter,
    pub name: String,
}

/// Configuration for retry behavior
#[derive(Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
        }
    }
}

/// Request configuration with headers
#[derive(Clone, Default)]
pub struct RequestConfig {
    pub headers: HashMap<String, String>,
    pub retry_config: Option<RetryConfig>,
}

impl RequestConfig {
    /// Create a new request config
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a header to the request
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add an API key header (X-API-Key)
    pub fn with_api_key(self, api_key: impl Into<String>) -> Self {
        self.with_header("X-API-Key", api_key)
    }

    /// Add a bearer token authorization header
    pub fn with_bearer_token(self, token: impl Into<String>) -> Self {
        self.with_header("Authorization", format!("Bearer {}", token.into()))
    }

    /// Add a basic auth header
    pub fn with_basic_auth(self, username: impl Into<String>, password: impl Into<String>) -> Self {
        let credentials = format!("{}:{}", username.into(), password.into());
        let encoded = base64::encode(credentials.as_bytes());
        self.with_header("Authorization", format!("Basic {}", encoded))
    }

    /// Add custom retry configuration
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = Some(config);
        self
    }
}

impl HttpClientManager {
    pub fn new() -> Self {
        // Create default client
        let default_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("media-collector/0.1.0")
            .build()
            .expect("Failed to create default HTTP client");

        // Create MyAnimeList client with custom settings
        let mal_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("media-collector/0.1.0")
            .build()
            .expect("Failed to create MyAnimeList HTTP client");

        Self {
            clients: Arc::new(ClientPool {
                default: ClientWithLimiter {
                    client: default_client,
                    limiter: RateLimiter::new("default", 10.0),
                    name: "default".to_string(),
                },
                my_anime_list: ClientWithLimiter {
                    client: mal_client,
                    limiter: RateLimiter::new("my_anime_list", 2.0),
                    name: "my_anime_list".to_string(),
                },
            }),
        }
    }

    /// Get the default HTTP client with rate limiter
    pub fn default(&self) -> &ClientWithLimiter {
        &self.clients.default
    }

    /// Get the MyAnimeList HTTP client with rate limiter
    pub fn my_anime_list(&self) -> &ClientWithLimiter {
        &self.clients.my_anime_list
    }
}

impl ClientWithLimiter {
    /// Execute a request with rate limiting
    /// The rate limiter will automatically wait if the limit is reached
    pub async fn execute<F, Fut, T>(&self, f: F) -> T
    where
        F: FnOnce(Client) -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        // Acquire rate limit permission (will wait if necessary)
        self.limiter.acquire().await;
        
        // Execute the request
        f(self.client.clone()).await
    }

    /// Try to execute a request immediately, return error if rate limited
    pub async fn try_execute<F, Fut, T>(&self, f: F) -> Result<T, std::time::Duration>
    where
        F: FnOnce(Client) -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        // Try to acquire without waiting
        self.limiter.try_acquire()?;
        
        // Execute the request
        Ok(f(self.client.clone()).await)
    }

    /// Fetch and deserialize JSON with automatic retry on rate limits
    /// 
    /// # Arguments
    /// * `url` - The URL to fetch
    /// * `config` - Optional request configuration with headers and retry settings
    /// 
    /// # Returns
    /// * `Ok(T)` - Successfully deserialized response
    /// * `Err(HttpError::NotFound)` - 404 response
    /// * `Err(HttpError::RateLimited)` - 429/403 after max retries
    /// * `Err(HttpError::*)` - Other errors
    pub async fn fetch_json<T: DeserializeOwned>(
        &self,
        url: &str,
        config: Option<RequestConfig>,
    ) -> Result<T, HttpError> {
        let config = config.unwrap_or_default();
        let retry_config = config.retry_config.unwrap_or_default();
        let mut attempt = 0;

        loop {
            attempt += 1;
            
            // Acquire rate limit permission
            self.limiter.acquire().await;
            
            println!("[{}] Fetching: {} (attempt {}/{})", 
                self.name, url, attempt, retry_config.max_retries + 1);

            // Build the request with headers
            let mut request = self.client.get(url);
            
            // Add custom headers
            for (key, value) in &config.headers {
                request = request.header(key, value);
            }

            // Make the request
            let response = match request.send().await {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("[{}] Request failed: {}", self.name, e);
                    return Err(HttpError::RequestFailed(e));
                }
            };

            let status = response.status();
            
            // Handle different status codes
            match status {
                StatusCode::OK => {
                    // Success - deserialize and return
                    return self.deserialize_response(response).await;
                }
                
                StatusCode::NOT_FOUND => {
                    // 404 - resource doesn't exist
                    let error_body = response.text().await
                        .unwrap_or_else(|_| "No error message".to_string());
                    
                    eprintln!("[{}] Resource not found (404): {}", self.name, url);
                    return Err(HttpError::NotFound(error_body));
                }
                
                StatusCode::TOO_MANY_REQUESTS | StatusCode::FORBIDDEN => {
                    // 429 or 403 - rate limited
                    let retry_after = self.parse_retry_after(&response);
                    let error_body = response.text().await
                        .unwrap_or_else(|_| "Rate limit exceeded".to_string());
                    
                    if attempt > retry_config.max_retries {
                        eprintln!("[{}] Max retries exceeded after rate limit", self.name);
                        return Err(HttpError::RateLimited {
                            retry_after,
                            message: error_body,
                        });
                    }
                    
                    // Calculate backoff delay
                    let delay = retry_after.unwrap_or_else(|| {
                        let exponential = retry_config.base_delay * 2_u32.pow(attempt - 1);
                        std::cmp::min(exponential, retry_config.max_delay)
                    });
                    
                    println!("[{}] Rate limited ({}), retrying in {:?}...", 
                        self.name, status.as_u16(), delay);
                    
                    tokio::time::sleep(delay).await;
                    continue;
                }
                
                _ => {
                    // Other status codes
                    let error_body = response.text().await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    
                    eprintln!("[{}] Unexpected status {}: {}", 
                        self.name, status.as_u16(), error_body);
                    
                    return Err(HttpError::UnexpectedStatus {
                        status: status.as_u16(),
                        message: error_body,
                    });
                }
            }
        }
    }

    /// Deserialize the response body to type T
    async fn deserialize_response<T: DeserializeOwned>(&self, response: Response) -> Result<T, HttpError> {
        response.json::<T>().await.map_err(|e| {
            eprintln!("[{}] Deserialization failed: {}", self.name, e);
            HttpError::DeserializationFailed(e.to_string())
        })
    }

    /// Parse the Retry-After header if present
    fn parse_retry_after(&self, response: &Response) -> Option<Duration> {
        response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| {
                // Try parsing as seconds (integer)
                s.parse::<u64>().ok().map(Duration::from_secs)
            })
    }
}