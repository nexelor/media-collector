use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use tracing::warn;

use crate::global::error::ConfigError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub app: AppSettings,
    pub database: DatabaseConfig,
    pub modules: ModulesConfig,
    pub child_modules: HashMap<String, ChildModuleConfig>,
    pub http: HttpConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppSettings {
    pub log_level: String,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_to_file")]
    pub log_to_file: bool,
    #[serde(default = "default_log_directory")]
    pub log_directory: String,
    #[serde(default = "default_log_file_prefix")]
    pub log_file_prefix: String,
    #[serde(default = "default_log_rotation")]
    pub log_rotation: LogRotation,
    #[serde(default = "default_log_to_console")]
    pub log_to_console: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogRotation {
    Daily,
    Hourly,
    Never,
}

fn default_log_to_file() -> bool {
    true
}

fn default_log_directory() -> String {
    "./logs".to_string()
}

fn default_log_file_prefix() -> String {
    "media-collector".to_string()
}

fn default_log_rotation() -> LogRotation {
    LogRotation::Daily
}

fn default_log_to_console() -> bool {
    true
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_to_file: default_log_to_file(),
            log_directory: default_log_directory(),
            log_file_prefix: default_log_file_prefix(),
            log_rotation: default_log_rotation(),
            log_to_console: default_log_to_console(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: i32,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModulesConfig {
    pub anime: ParentModuleConfig,
    #[serde(default)]
    pub manga: ParentModuleConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParentModuleConfig {
    pub enabled: bool,
}

impl Default for ParentModuleConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChildModuleConfig {
    pub enabled: bool,
    pub rate_limit: f64,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub requires_api_key: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpConfig {
    pub timeout_seconds: u64,
    pub user_agent: String,
    pub default_rate_limit: f64,
    pub retry: RetryConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl AppConfig {
    /// Load configuration from config.toml file
    pub fn load() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .build()
            .map_err(|e| ConfigError::LoadFailed(e.to_string()))?;

        let app_config: AppConfig = config.try_deserialize()
            .map_err(|e| ConfigError::LoadFailed(e.to_string()))?;

        Ok(app_config)
    }

    /// Validate that a child module has required configuration
    /// Returns Ok(()) if valid, Err(ConfigError) if missing required config
    pub fn validate_child_module(&self, module_name: &str, requires_api_key: bool) -> Result<(), ConfigError> {
        let config = self.child_modules.get(module_name)
            .ok_or_else(|| ConfigError::Invalid(format!("Module '{}' not found in configuration", module_name)))?;

        if !config.enabled {
            return Err(ConfigError::Invalid(format!("Module '{}' is disabled", module_name)));
        }

        if requires_api_key && config.api_key.is_empty() {
            return Err(ConfigError::MissingApiKey(module_name.to_string()));
        }

        Ok(())
    }

    /// Check if a child module is properly configured and can be started
    /// Returns true only if enabled and has all required configuration
    pub fn can_start_child_module(&self, module_name: &str, requires_api_key: bool) -> bool {
        match self.validate_child_module(module_name, requires_api_key) {
            Ok(_) => true,
            Err(e) => {
                warn!(
                    module = %module_name,
                    error = %e,
                    "Child module cannot start due to configuration issue"
                );
                false
            }
        }
    }

    /// Check if a parent module is enabled
    pub fn is_parent_module_enabled(&self, module_name: &str) -> bool {
        match module_name {
            "anime" => self.modules.anime.enabled,
            "manga" => self.modules.manga.enabled,
            _ => false,
        }
    }

    /// Check if a child module is enabled
    pub fn is_child_module_enabled(&self, module_name: &str) -> bool {
        self.child_modules
            .get(module_name)
            .map(|config| config.enabled)
            .unwrap_or(false)
    }

    /// Get child module configuration
    pub fn get_child_module_config(&self, module_name: &str) -> Option<&ChildModuleConfig> {
        self.child_modules.get(module_name)
    }

    /// Get rate limit for a child module
    pub fn get_rate_limit(&self, module_name: &str) -> f64 {
        self.child_modules
            .get(module_name)
            .map(|config| config.rate_limit)
            .unwrap_or(self.http.default_rate_limit)
    }

    /// Get API key for a child module
    pub fn get_api_key(&self, module_name: &str) -> Option<String> {
        self.child_modules
            .get(module_name)
            .and_then(|config| {
                if config.api_key.is_empty() {
                    None
                } else {
                    Some(config.api_key.clone())
                }
            })
    }
}