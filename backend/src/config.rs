//! 配置管理模块
//! 从 config.yaml 加载配置，支持环境变量覆盖

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: u64,
}

impl Config {
    /// 从配置文件加载配置
    /// 优先级：环境变量 > config.yaml
    pub fn load() -> anyhow::Result<Self> {
        // 直接使用绝对路径加载配置
        let config_path = "/Users/mac/projects/rust/nothing-todo-cloud/backend/config.yaml";

        let settings = config::Config::builder()
            .add_source(config::File::with_name(config_path))
            .add_source(config::Environment::with_prefix("NTD").separator("__"))
            .build()?;

        let config: Config = settings.try_deserialize()?;
        eprintln!("数据库URL: {}", config.database.url);
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
            },
            database: DatabaseConfig {
                url: "sqlite:ntd_cloud.db".to_string(),
            },
            jwt: JwtConfig {
                secret: "change-me-in-production".to_string(),
                expiration_hours: 24 * 7, // 7 天
            },
        }
    }
}
