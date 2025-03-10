use core::fmt;

/// Represents different types of Stellar accounts.
#[derive(Debug)]
pub enum AccountType {
    /// Smart Contract
    Contract,
    /// N/M Multisig account
    Multisig(u8, u8),
    /// Account with no active signers
    Deactivated,
    /// Hot wallet (single signer with full control)
    HotWallet,
    /// Multi-Party Computation wallet
    MPC,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccountType::Contract => write!(f, "Contract"),
            AccountType::Deactivated => write!(f, "Deactivated Account"),
            AccountType::Multisig(threshold, total) => {
                write!(f, "Multisig {}/{}", threshold, total)
            }
            AccountType::HotWallet => write!(f, "Hot Wallet"),
            AccountType::MPC => write!(f, "MPC"),
        }
    }
}
