use thiserror::Error;

#[derive(Debug, Error, Clone)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Error {
    #[error("multiple potential admin addresses")]
    MultipleAdminsFound,
    #[error("wrong storage type")]
    WrongStorageType,
    #[error("malformed address")]
    MalformedAddress,
    #[error("malformed rpc url")]
    MalformedUrl,
    #[error("admin not found")]
    AdminNotFound,
    #[error("target address is not a contract")]
    NotAContract,
    #[error("failed to fetch instance storage")]
    InstanceStorageFailure,
    #[error("failed to fetch persistent storage")]
    PersistentStorageFailure,
    #[error("unknown network")]
    InvalidNetwork,
    #[error("failed to load config")]
    ConfigLoadFailure,
    #[error("XDG_CONFIG_HOME env variable is not a valid path. Got {0}")]
    XdgConfigHome(String),
    #[error("failed to find home directory")]
    HomeDirNotFound,
    #[error("rpc url not set in config")]
    RpcUrlNotSet,
    #[error("failed to parse toml")]
    TomlParseFailure,
    #[error("Contract id or admin is missing")]
    MissingTargetAddress,
    #[error("missing network")]
    MissingNetwork,
    #[error("cannot find horizon url")]
    HorizonUrlNotAvailable,
    #[error("failed to fetch horizon data")]
    HorizonDataFetchFailure,
    #[error("failed to parse horizon data json")]
    HorizonDataParseFailure,
}
