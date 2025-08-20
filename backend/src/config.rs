use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub openai_api_key: Option<String>,
    pub claude_api_key: Option<String>,
    pub github_token: Option<String>,
    pub stripe_secret_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Config {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()?,
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://maxamem:password@localhost/maxamem".to_string()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string()),
            openai_api_key: env::var("OPENAI_API_KEY").ok(),
            claude_api_key: env::var("CLAUDE_API_KEY").ok(),
            github_token: env::var("GITHUB_TOKEN").ok(),
            stripe_secret_key: env::var("STRIPE_SECRET_KEY").ok(),
        })
    }
}