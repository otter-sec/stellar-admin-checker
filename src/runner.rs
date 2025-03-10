use crate::{account_type::AccountType, error::Error, horizon_helper, storage_helper};
use std::{collections::HashSet, str::FromStr};
use stellar_rpc_client::Client;
use stellar_xdr::curr::{
    ContractDataDurability, LedgerEntryData, LedgerKey, LedgerKeyContractData, Limits, ReadXdr,
    ScAddress, ScMapEntry, ScVal,
};

use crate::storage_helper::{decode_admin_value, possible_keys};

/// Runner handles the core logic
pub struct Runner {
    rpc: Client,
    horizon: String,
    contract_id: ScAddress,
    keys: HashSet<ScVal>,
}

impl Runner {
    /// Creates a new Runner instance with the specified configuration.
    ///
    /// # Arguments
    /// * `rpc_url` - The RPC endpoint to use
    /// * `horizon` - The Horizon URL to use
    /// * `contract_id` - The ID of the contract to analyze
    /// * `key` - The admin address' storage key
    pub fn new(
        rpc_url: &str,
        horizon: String,
        contract_id: &str,
        key: &str,
    ) -> Result<Self, Error> {
        Ok(Self {
            rpc: Client::new(rpc_url).map_err(|_| Error::MalformedUrl)?,
            horizon,
            contract_id: ScAddress::from_str(&contract_id).map_err(|_| Error::MalformedAddress)?,
            keys: possible_keys(mutate_input(key)),
        })
    }

    /// Finds the admin key in contract storage or returns the EOA address.
    ///
    /// For EOAs, returns the address directly. For contracts, searches both
    /// instance and persistent storage for the admin key.
    pub async fn find_key(&self) -> Result<storage_helper::AddressType, Error> {
        if let ScAddress::Account(id) = self.contract_id.clone() {
            return Ok(storage_helper::wrap_eoa(id));
        }

        let instance_storage = self.get_contract_instance().await?;

        let admin_val = if let Some(entry) = instance_storage
            .iter()
            .find(|entry| self.keys.contains(&entry.key))
        {
            entry.val.clone()
        } else {
            self.persistent_storage_lookup().await?
        };

        decode_admin_value(&admin_val)
    }

    /// Determines if an EOA is a hot wallet or MPC based on transaction patterns.
    ///
    /// # Arguments
    /// * `admin_address` - The address of the EOA to analyze
    ///
    /// Returns the account type based on:
    /// - Signer weights and thresholds
    /// - Transaction frequency patterns
    pub async fn is_hot_wallet(&self, admin_address: String) -> Result<AccountType, Error> {
        let account_type =
            horizon_helper::check_if_centralized(self.horizon.clone(), admin_address.clone())
                .await?;

        match account_type {
            AccountType::HotWallet => {
                let min_ledger_diff_between_txs =
                    horizon_helper::tx_frequency_for_account(self.horizon.clone(), admin_address)
                        .await?;
                // If there's less than 12 ledgers (1 min) between transactions, it's likely a hot wallet
                if min_ledger_diff_between_txs <= 12 {
                    Ok(AccountType::HotWallet)
                } else {
                    Ok(AccountType::MPC)
                }
            }
            _ => Ok(account_type),
        }
    }

    /// Looks up the admin key in persistent contract storage.
    ///
    /// Used in case the admin key is not found in the instance storage.
    async fn persistent_storage_lookup(&self) -> Result<ScVal, Error> {
        let result = self
            .rpc
            .get_ledger_entries(&self.persistent_storage_keys())
            .await;

        if let Ok(entries_) = result {
            let entries = entries_.entries.unwrap();

            match entries.len() {
                0 => return Err(Error::AdminNotFound),
                1 => (),
                _ => return Err(Error::MultipleAdminsFound),
            }

            let entry = entries.get(0).unwrap();
            let val = LedgerEntryData::from_xdr_base64(entry.xdr.clone(), Limits::none()).unwrap();
            if let LedgerEntryData::ContractData(data) = val {
                Ok(data.val)
            } else {
                Err(Error::AdminNotFound)
            }
        } else {
            Err(Error::PersistentStorageFailure)
        }
    }

    /// Retrieves the contract instance storage.
    async fn get_contract_instance(&self) -> Result<Vec<ScMapEntry>, Error> {
        if let ScAddress::Contract(hash) = &self.contract_id {
            let instance = self
                .rpc
                .get_contract_instance(&hash.0)
                .await
                .map_err(|_| Error::InstanceStorageFailure)?;
            if let Some(storage) = instance.storage {
                Ok(storage.0.to_vec())
            } else {
                Ok(vec![])
            }
        } else {
            Err(Error::NotAContract)
        }
    }

    /// Generates ledger keys for persistent storage lookup.
    fn persistent_storage_keys(&self) -> Vec<LedgerKey> {
        self.keys
            .iter()
            .map(|k| {
                LedgerKey::ContractData(LedgerKeyContractData {
                    contract: self.contract_id.clone(),
                    key: k.clone(),
                    durability: ContractDataDurability::Persistent,
                })
            })
            .collect()
    }
}

/// Generates variations of the key for storage lookup.
fn mutate_input(key: &str) -> Vec<String> {
    let capitalized = key
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect();

    vec![
        key.to_ascii_lowercase(),
        key.to_ascii_uppercase(),
        capitalized,
    ]
}
