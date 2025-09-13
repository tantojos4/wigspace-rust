impl Default for Config {
    fn default() -> Self {
        Config {
            address: "127.0.0.1".to_string(),
            port: 8080,
            static_dir: None,
            proxy_pass: None,
            access_log: None,
            error_log: None,
            plugins_dir: Some("plugins".to_string()),
            plugin_endpoints: None,
        }
    }
}
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub address: String,
    pub port: u16,
    pub static_dir: Option<String>,
    pub proxy_pass: Option<String>,
    pub access_log: Option<String>,
    pub error_log: Option<String>,
    pub plugins_dir: Option<String>,
    pub plugin_endpoints: Option<std::collections::HashMap<String, String>>,
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Config {
    let content = fs::read_to_string(path).expect("Failed to read config file");
    serde_yaml::from_str(&content).expect("Failed to parse config file")
}
