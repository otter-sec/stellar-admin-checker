use crate::Error;
use std::str::FromStr;
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub enum UrlType {
    Rpc,
    Horizon,
}

/// Retrieves the appropriate URL for a given network and URL type.
///
/// # Arguments
/// * `network` - The network name (e.g., "mainnet", "testnet")
/// * `url_type` - The type of URL to retrieve
pub fn url_for_network(network: &str, url_type: UrlType) -> Result<String, Error> {
    match url_type {
        UrlType::Rpc => {
            if let Ok(rpc_url) = rpc_url_from_network(&network) {
                Ok(rpc_url)
            } else {
                // If it's not a "well-known" network, try to load it from local config file
                load_from_config(&network)
            }
        }
        UrlType::Horizon => horizon_url_from_network(network),
    }
}

/// Hardcoded RPC URLs for well-known networks
///
/// # Arguments
/// * `network` - The network name
fn rpc_url_from_network(network: &str) -> Result<String, Error> {
    match network {
        "mainnet" => Ok("https://mainnet.sorobanrpc.com".to_string()),
        "testnet" => Ok("https://soroban-testnet.stellar.org".to_string()),
        "futurenet" => Ok("https://rpc-futurenet.stellar.org".to_string()),
        "local" | "standalone" => Ok("http://localhost:8000/soroban/rpc".to_string()),
        _ => Err(Error::InvalidNetwork),
    }
}

/// Hardcoded Horizon URLs for well-known networks
///
/// # Arguments
/// * `network` - The network name
fn horizon_url_from_network(network: &str) -> Result<String, Error> {
    match network {
        "testnet" => Ok("https://horizon-testnet.stellar.org/".to_string()),
        "futurenet" => Ok("https://horizon-futurenet.stellar.org/".to_string()),
        "mainnet" => Ok("https://horizon.stellar.org/".to_string()),
        "local" | "standalone" => Err(Error::HorizonUrlNotAvailable),
        _ => Err(Error::InvalidNetwork),
    }
}

/// Loads network configuration from local config files
///
/// # Arguments
/// * `network_file` - The network configuration file name
fn load_from_config(network_file: &str) -> Result<String, Error> {
    let config_dir = if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from_str(&config_home).map_err(|_| Error::XdgConfigHome(config_home))?
    } else {
        dirs::home_dir()
            .ok_or(Error::HomeDirNotFound)?
            .join(".config")
    };

    let soroban_dir = config_dir.join("soroban/network").join(network_file);
    let stellar_dir = config_dir.join("stellar/network").join(network_file);

    match (stellar_dir.exists(), soroban_dir.exists()) {
        (true, _) => rpc_url_from_toml(stellar_dir),
        (false, true) => rpc_url_from_toml(soroban_dir),
        _ => Err(Error::ConfigLoadFailure),
    }
}

/// Extracts the RPC URL from TOML
///
/// # Arguments
/// * `path` - Path to the TOML configuration file
fn rpc_url_from_toml(path: PathBuf) -> Result<String, Error> {
    let toml_content = fs::read_to_string(path).map_err(|_| Error::ConfigLoadFailure)?;

    match toml_content.parse::<toml::Value>() {
        Ok(toml_value) => {
            if let Some(rpc_url) = toml_value.get("rpc_url").and_then(|v| v.as_str()) {
                Ok(rpc_url.to_string())
            } else {
                Err(Error::RpcUrlNotSet)
            }
        }
        _ => Err(Error::TomlParseFailure),
    }
}
