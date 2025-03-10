use crate::{account_type::AccountType, error::Error};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Thresholds {
    pub low_threshold: u8,
}

#[derive(Deserialize, Debug)]
struct Signer {
    pub weight: u8,
}

#[derive(Deserialize, Debug)]
struct AccountData {
    pub thresholds: Thresholds,
    pub signers: Vec<Signer>,
}

#[derive(Deserialize, Debug, Clone)]
struct TxRecord {
    pub ledger: u64,
    pub paging_token: String,
    pub source_account: String,
    pub fee_account: String,
}

#[derive(Deserialize, Debug)]
struct Response {
    pub records: Vec<TxRecord>,
}

#[derive(Deserialize, Debug)]
struct LinkObject {
    pub href: String,
}

#[derive(Deserialize, Debug)]
struct Links {
    pub next: LinkObject,
}

#[derive(Deserialize, Debug)]
struct Wrapper {
    pub _links: Links,
    pub _embedded: Response,
}

/// Checks if a "G" account is a multisig/"decentralized" account by analyzing its signers and thresholds.
///
/// Returns:
/// - `AccountType::HotWallet` if any single signer has enough weight for transactions
/// - `AccountType::Multisig` if multiple signers are required
/// - `AccountType::Deactivated` if the account has no active signers
///
/// # Arguments
/// * `horizon_url` - The base URL of the Horizon API
/// * `account_id` - The Stellar account ID to check
pub async fn check_if_centralized(
    horizon_url: String,
    account_id: String,
) -> Result<AccountType, Error> {
    let url = format!("{}accounts/{}/", horizon_url, account_id);
    let response = reqwest::get(&url)
        .await
        .map_err(|_| Error::HorizonDataFetchFailure)?;

    if !response.status().is_success() {
        return Err(Error::HorizonDataFetchFailure);
    }

    let body: AccountData = response
        .json()
        .await
        .map_err(|_| Error::HorizonDataParseFailure)?;

    let mut weights: Vec<u8> = body.signers.iter().map(|s| s.weight).collect();
    let max_weight = *weights.iter().max().unwrap_or(&0);

    if max_weight == 0 {
        return Ok(AccountType::Deactivated);
    }

    if max_weight >= body.thresholds.low_threshold {
        return Ok(AccountType::HotWallet);
    }

    // Determine multisig account type
    let total_signers = weights.iter().filter(|&&x| x > 0).count();
    weights.sort_unstable_by(|a, b| b.cmp(a));

    let mut total_weight = 0;
    for i in 0..total_signers {
        total_weight += weights[i];
        if total_weight >= body.thresholds.low_threshold {
            return Ok(AccountType::Multisig(i as u8 + 1, total_signers as u8));
        }
    }

    Ok(AccountType::Deactivated)
}

/// Calculates the minimum time between transactions for an account.
///
/// Returns the minimum number of ledgers between any two consecutive transactions.
/// Returns `u64::MAX` if the account has fewer than 2 transactions.
///
/// # Arguments
/// * `horizon_url` - The base URL of the Horizon API
/// * `account_id` - The Stellar account ID to analyze
pub async fn tx_frequency_for_account(
    horizon_url: String,
    account_id: String,
) -> Result<u64, Error> {
    let txs = get_all_txs_for_account(horizon_url.clone(), account_id.clone()).await?;

    if txs.len() < 2 {
        return Ok(u64::MAX);
    }

    let min_ledger_dif = txs
        .windows(2)
        .map(|r| r[1].ledger - r[0].ledger)
        .min()
        .unwrap();

    Ok(min_ledger_dif)
}

async fn get_all_txs_for_account(
    horizon_url: String,
    account_id: String,
) -> Result<Vec<TxRecord>, Error> {
    let mut results = Vec::new();
    let mut next_url = None;

    loop {
        let (txs, next_url_) =
            get_txs_from_cursor(horizon_url.clone(), account_id.clone(), next_url).await?;
        results.extend(txs.clone());

        if txs.len() != 200 {
            break;
        }

        next_url = Some(next_url_);
    }

    results.retain(|r| r.source_account == account_id || r.fee_account == account_id);
    results.dedup_by_key(|r| r.paging_token.clone());
    Ok(results)
}

async fn get_txs_from_cursor(
    horizon_url: String,
    account_id: String,
    url_: Option<String>,
) -> Result<(Vec<TxRecord>, String), Error> {
    let url = url_.unwrap_or(format!(
        "{}accounts/{}/transactions?limit=200&include_failed=false",
        horizon_url, account_id
    ));

    let response = reqwest::get(&url)
        .await
        .map_err(|_| Error::HorizonDataFetchFailure)?;

    if !response.status().is_success() {
        return Err(Error::HorizonDataFetchFailure);
    }

    let body: Wrapper = response
        .json()
        .await
        .map_err(|_| Error::HorizonDataParseFailure)?;

    Ok((body._embedded.records, body._links.next.href))
}
