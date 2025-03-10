//! Admin Wallet Type Checker
//!
//! Analyzes Stellar accounts and contracts to determine their type:
//! - For contracts: Identifies if it's a contract account
//! - For EOAs: Determines if it's a hot wallet, MPC, or multisig account

mod account_type;
mod config;
mod error;
mod horizon_helper;
mod network_config;
mod runner;
mod storage_helper;
use account_type::AccountType;
use clap::CommandFactory;
use config::Config;
use error::Error;
use storage_helper::AddressType;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Error> {
    if std::env::args().len() == 1 {
        Config::command().print_help().unwrap();
        return Ok(());
    }

    let config = Config::parce_args()?;
    let runner = config.to_runner()?;

    let account_type = match runner.find_key().await? {
        AddressType::EOA(addr) => runner.is_hot_wallet(addr).await?,
        AddressType::Contract => AccountType::Contract,
    };

    println!("Account type: {}", account_type);
    Ok(())
}
