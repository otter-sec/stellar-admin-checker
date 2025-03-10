use std::collections::HashSet;
use std::str::FromStr;
use stellar_xdr::curr::{AccountId, ScAddress, ScString, ScSymbol, ScVal, ScVec, StringM};

use crate::error::Error;

#[derive(Debug)]
pub enum AddressType {
    EOA(String),
    Contract,
}

/// Possible formats for the admin storage key
#[derive(Debug)]
enum KeyType {
    EnumVariant,
    Symbol,
    String,
}

/// Generates a set of possible storage keys for contract lookup.
///
/// For each input key, generates variations in all possible formats:
/// - Enum variant format
/// - Symbol format
/// - String format
///
/// # Arguments
/// * `keys` - Vector of key strings to generate variations for
pub fn possible_keys(keys: Vec<String>) -> HashSet<ScVal> {
    let mut ret = HashSet::new();
    for k in keys {
        ret.insert(format_key(&k, KeyType::EnumVariant));
        ret.insert(format_key(&k, KeyType::String));
        ret.insert(format_key(&k, KeyType::Symbol));
    }
    ret
}

/// Wraps an AccountId into an AddressType
///
/// # Arguments
/// * `id` - The AccountId to wrap
pub fn wrap_eoa(id: AccountId) -> AddressType {
    AddressType::EOA(AccountId::to_string(&id))
}

/// Converts ScVal into AddressType
///
/// # Arguments
/// * `val` - The ScVal to decode
pub fn decode_admin_value(val: &ScVal) -> Result<AddressType, Error> {
    let addr = match val {
        ScVal::Address(addr) => addr.clone(),
        _ => return Err(Error::WrongStorageType),
    };

    if let ScAddress::Account(id) = addr {
        Ok(wrap_eoa(id))
    } else {
        Ok(AddressType::Contract)
    }
}

/// Formats a key string into the specified ScVal format.
///
/// # Arguments
/// * `key` - The key string to format
/// * `key_type` - The desired format type
fn format_key(key: &str, key_type: KeyType) -> ScVal {
    match key_type {
        KeyType::EnumVariant => get_enum_variant_key(key),
        KeyType::Symbol => ScVal::Symbol(ScSymbol::from(StringM::from_str(key).unwrap())),
        KeyType::String => ScVal::String(ScString::from(StringM::from_str(key).unwrap())),
    }
}

fn get_enum_variant_key(key: &str) -> ScVal {
    ScVal::Vec(Some(
        ScVec::try_from(vec![ScVal::Symbol(ScSymbol::from(ScSymbol::from(
            StringM::from_str(key).unwrap(),
        )))])
        .unwrap(),
    ))
}
