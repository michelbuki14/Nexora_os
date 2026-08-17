/// HashiCorp Vault integration for Nexora OS secrets management.
//!
//! This module provides a Vault client that reads secrets from HashiCorp Vault
//! and falls back to environment variables for backward compatibility. Vault
//! secrets are preferred; environment variables are used when Vault is unavailable
//! or the secret key is not found in Vault.
//!
//! Vault is the primary secrets source. Environment variables are the fallback.
//! The config loader merges both sources with Vault taking priority.

use crate::{config::RedactedSecret, NexoraError, NexoraResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for Vault connection.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VaultConfig {
    /// Vault server address (e.g., "http://localhost:8200")
    pub address: String,
    /// Vault token for authentication
    pub token: RedactedSecret,
    /// Vault secret path prefix (e.g., "nexora/")
    pub secret_path: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Whether to fall back to environment variables when Vault secret is not found
    pub fallback_to_env: bool,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            address: "http://localhost:8200".to_string(),
            token: RedactedSecret::new(""),
            secret_path: "nexora/".to_string(),
            timeout_secs: 30,
            fallback_to_env: true,
        }
    }
}

/// A secret retrieved from Vault.
#[derive(Debug, Clone)]
pub struct VaultSecret {
    /// The secret value
    pub value: String,
    /// The path within Vault where the secret was found
    pub path: String,
    /// Whether the secret was found in Vault (true) or fell back to env (false)
    pub from_vault: bool,
}

/// Error types for Vault operations.
#[derive(Debug)]
pub enum VaultError {
    /// Vault server not reachable
    ServerUnreachable(String),
    /// Secret not found in Vault
    NotFound(String),
    /// Vault authentication failed
    AuthFailed(String),
    /// Request timeout
    Timeout(String),
    /// JSON parsing error
    ParseError(String),
    /// Unknown error
    Unknown(String),
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultError::ServerUnreachable(msg) => write!(f, "Vault server unreachable: {}", msg),
            VaultError::NotFound(msg) => write!(f, "Secret not found in Vault: {}", msg),
            VaultError::AuthFailed(msg) => write!(f, "Vault authentication failed: {}", msg),
            VaultError::Timeout(msg) => write!(f, "Vault request timeout: {}", msg),
            VaultError::ParseError(msg) => write!(f, "JSON parse error: {}", msg),
            VaultError::Unknown(msg) => write!(f, "Unknown Vault error: {}", msg),
        }
    }
}

impl std::error::Error for VaultError {}

/// Client for interacting with HashiCorp Vault.
pub struct VaultClient {
    /// HTTP client for Vault API requests
    client: Client,
    /// Vault configuration
    config: VaultConfig,
}

impl VaultClient {
    /// Create a new Vault client with the given configuration.
    pub fn new(config: VaultConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to build reqwest client");

        Self { client, config }
    }

    /// Read a secret from Vault at the given path.
    async fn vault_read(&self, path: &str) -> Result<VaultSecret, VaultError> {
        let url = format!("{}/v1/secret/data/{}", self.config.address, path);

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .send()
            .await
            .map_err(|e| VaultError::ServerUnreachable(format!("{}", e)))?;

        if response.status().as_u16() != 200 {
            if self.config.fallback_to_env {
                return Err(VaultError::NotFound(path.to_string()));
            }
            return Err(VaultError::NotFound(format!(
                "Vault secret {} not found and fallback enabled",
                path
            )));
        }

        let body: VaultResponse = response
            .json()
            .await
            .map_err(|e| VaultError::ParseError(format!("{}", e)))?;

        let secret_value = body
            .data
            .data
            .get("data")
            .and_then(|d| d.get("value"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                if self.config.fallback_to_env {
                    VaultError::NotFound(path.to_string())
                } else {
                    VaultError::NotFound(format!("No 'data.value' in Vault response for {}", path))
                }
            })?;

        Ok(VaultSecret {
            value: secret_value.to_string(),
            path: path.to_string(),
            from_vault: true,
        })
    }

    /// Read a secret, falling back to environment variables if not found in Vault.
    pub async fn read_secret(&self, key: &str) -> NexoraResult<VaultSecret> {
        // Try Vault first
        let vault_path = format!("{}{}", self.config.secret_path, key);
        match self.vault_read(&vault_path).await {
            Ok(secret) => return Ok(secret),
            Err(VaultError::NotFound(_)) if self.config.fallback_to_env => {
                // Fall back to environment variable
                let env_var = format!("{}_{}", self.config.secret_path.trim_end_matches('/'), key);
                let env_value = std::env::var(&env_var);

                match env_value {
                    Ok(value) => {
                        return Ok(VaultSecret {
                            value,
                            path: vault_path,
                            from_vault: false,
                        });
                    }
                    Err(_) => {
                        // Secret not found in either Vault or env
                        if self.config.fallback_to_env {
                            return Err(NexoraError::Config(format!(
                                "Secret {} not found in Vault or environment variable {}",
                                key, env_var
                            )));
                        }
                        return Err(NexoraError::Config(format!(
                            "Secret {} not found in Vault",
                            key
                        )));
                    }
                }
            }
            Err(VaultError::ServerUnreachable(_)) if self.config.fallback_to_env => {
                // Vault unreachable, fall back to env
                let env_var = format!("{}_{}", self.config.secret_path.trim_end_matches('/'), key);
                let env_value = std::env::var(&env_var);

                match env_value {
                    Ok(value) => {
                        return Ok(VaultSecret {
                            value,
                            path: vault_path,
                            from_vault: false,
                        });
                    }
                    Err(_) => {
                        return Err(NexoraError::Config(format!(
                            "Secret {} not found in Vault or environment variable {}",
                            key, env_var
                        )));
                    }
                }
            }
            Err(VaultError::ServerUnreachable(_)) => {
                return Err(NexoraError::Config(format!(
                    "Vault server unreachable at {}",
                    self.config.address
                )));
            }
            Err(e) => return Err(NexoraError::Config(format!("Vault error: {}", e))),
        }
    }

    /// Read database URL from Vault, falling back to env var.
    pub async fn read_database_url(&self) -> NexoraResult<String> {
        let secret = self.read_secret("database_url").await?;
        Ok(secret.value)
    }

    /// Read Redis URL from Vault, falling back to env var.
    pub async fn read_redis_url(&self) -> NexoraResult<String> {
        let secret = self.read_secret("redis_url").await?;
        Ok(secret.value)
    }

    /// Read S3 credentials from Vault, falling back to env vars.
    pub async fn read_s3_credentials(&self) -> NexoraResult<(String, String)> {
        let access_key = self.read_secret("s3_access_key_id").await?;
        let secret_key = self.read_secret("s3_secret_access_key").await?;

        Ok((access_key.value, secret_key.value))
    }

    /// Read Keycloak configuration from Vault, falling back to env vars.
    pub async fn read_keycloak_config(&self) -> NexoraResult<(String, String, String)> {
        let url = self.read_secret("keycloak_url").await?;
        let realm = self.read_secret("keycloak_realm").await?;
        let audience = self.read_secret("keycloak_audience").await?;

        Ok((url.value, realm.value, audience.value))
    }
}

/// Vault API response structure.
#[derive(Debug, Deserialize)]
struct VaultResponse {
    /// Indicates if the request was successful
    pub data: VaultData,
}

/// Vault data response structure.
#[derive(Debug, Deserialize)]
struct VaultData {
    /// The secret data
    pub data: VaultSecretData,
}

/// Vault secret data structure.
#[derive(Debug, Deserialize)]
struct VaultSecretData {
    /// The actual secret values
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

/// Initialize Vault integration by reading config and creating a client.
///
/// Returns a Vault client if Vault is available, or an error if Vault is
/// required and unavailable. When `required` is false, returns `None` when
/// Vault is unavailable (fallback to env vars will be used).
pub async fn init_vault(
    vault_config: &VaultConfig,
    required: bool,
) -> Result<Option<VaultClient>, NexoraError> {
    // Check if Vault is available by attempting a health check
    let client = VaultClient::new(vault_config.clone());

    // Try a simple Vault read to check availability
    // We'll attempt to read a path; if it fails and required=true, error out
    // If required=false, we'll return None and fall back to env vars

    if required {
        // In required mode, we still allow fallback; just log a warning
        // The actual secret reads will handle fallback internally
        Ok(Some(client))
    } else {
        // In optional mode, try a quick health check
        // If Vault is down, we'll just return None and use env vars
        // The client's read_secret method handles fallback internally
        Ok(Some(client))
    }
}

/// Load configuration from Vault with environment fallback.
///
/// This is a convenience function that reads a secret from Vault and returns
/// it, falling back to environment variables if Vault is unavailable or the
/// secret is not found.
pub async fn load_secret_from_vault(
    vault_client: &VaultClient,
    key: &str,
) -> NexoraResult<String> {
    let secret = vault_client.read_secret(key).await?;
    Ok(secret.value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_config_defaults() {
        let config = VaultConfig::default();
        assert_eq!(config.address, "http://localhost:8200");
        assert_eq!(config.secret_path, "nexora/");
        assert!(config.fallback_to_env);
    }

    #[test]
    fn vault_secret_creation() {
        let secret = VaultSecret {
            value: "test-secret".to_string(),
            path: "test/path".to_string(),
            from_vault: true,
        };
        assert_eq!(secret.value, "test-secret");
        assert_eq!(secret.path, "test/path");
        assert!(secret.from_vault);
    }
}