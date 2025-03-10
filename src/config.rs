use crate::{
    network_config::{url_for_network, UrlType},
    runner::Runner,
    Error,
};
use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(version = "0.1", about = "Checks admin wallet type")]
pub struct Config {
    #[arg(short, long, conflicts_with("admin"), help("Target Contract ID"))]
    contract_id: Option<String>,
    #[arg(
        short,
        long,
        conflicts_with_all(&["contract_id", "key"]),
        help("Admin key to search for. When used, all the other options are ignored")
    )]
    admin: Option<String>,
    #[arg(
        short,
        long,
        conflicts_with("network"),
        help("RPC URL to use. If want to use a known or imported network, use the 'network' option instead")
    )]
    rpc_url: Option<String>,
    #[arg(short, long, conflicts_with("rpc_url"), help("Network to use. Available options are 'mainnet', 'testnet', 'futurenet', 'local' or 'standalone'. Can also be used with custom networks, as long as these are imported in the local stellar-cli config"))]
    network: Option<String>,
    #[arg(
        short,
        long,
        default_value("admin"),
        help("Admin's storage slot key to search for. Defaults to 'admin'")
    )]
    key: String,
    #[arg(
        long,
        help("Horizon URL to use. If not provided, it will be inferred from the network")
    )]
    horizon: Option<String>,
}

impl Config {
    /// Parses command line arguments and validates the configuration.
    pub fn parce_args() -> Result<Config, Error> {
        let mut config = Config::parse();

        if config.network.is_none()
            && config.rpc_url.is_none()
            && env::var("SOROBAN_NETWORK").is_err()
        {
            return Err(Error::MissingNetwork);
        }

        if config.network.is_none() {
            config.network = env::var("SOROBAN_NETWORK").ok()
        }

        if config.admin.is_none() && config.contract_id.is_none() {
            return Err(Error::MissingTargetAddress);
        }

        Ok(config)
    }

    /// Creates a Runner instance with the current configuration
    pub fn to_runner(&self) -> Result<Runner, Error> {
        let contract_id = if self.admin.is_some() {
            self.admin.clone()
        } else {
            self.contract_id.clone()
        }
        .ok_or(Error::MissingTargetAddress)?;

        if self.network.is_none() && self.horizon.is_none() {
            return Err(Error::HorizonUrlNotAvailable);
        }

        let network = if let Some(n) = &self.network {
            Ok(n.clone())
        } else {
            env::var("SOROBAN_NETWORK").map_err(|_| Error::MissingNetwork)
        };

        let rpc_url = self.get_url(&network, UrlType::Rpc)?;
        let horizon_url = self.get_url(&network, UrlType::Horizon)?;

        Runner::new(&rpc_url, horizon_url, &contract_id, &self.key)
    }

    /// Gets the appropriate URL for the specified network and URL type.
    ///
    /// # Arguments
    /// * `network` - The network configuration
    /// * `url_type` - The type of URL to retrieve
    fn get_url(&self, network: &Result<String, Error>, url_type: UrlType) -> Result<String, Error> {
        let mut url = match url_type {
            UrlType::Rpc => {
                if let Some(url) = &self.rpc_url {
                    url.clone()
                } else {
                    url_for_network(&network.clone()?, UrlType::Rpc)?
                }
            }
            UrlType::Horizon => {
                if let Some(url) = &self.horizon {
                    url.clone()
                } else {
                    url_for_network(&network.clone()?, UrlType::Horizon)?
                }
            }
        };
        if !url.ends_with('/') {
            url.push('/');
        }
        Ok(url)
    }
}
