use std::fmt::{self, Debug, Display, Formatter};

use hex_fmt::HexFmt;
use serde::{Deserialize, Serialize};

use types::{
    bytesrepr,
    contracts::{ContractVersion, DEFAULT_ENTRY_POINT_NAME},
    ContractHash, ContractPackageHash, Key, RuntimeArgs,
};

use super::error;
use crate::components::contract_runtime::{core::execution, shared::account::Account};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ExecutableDeployItem {
    ModuleBytes {
        module_bytes: Vec<u8>,
        // assumes implicit `call` noarg entrypoint
        args: Vec<u8>,
    },
    StoredContractByHash {
        hash: ContractHash,
        entry_point: String,
        args: Vec<u8>,
    },
    StoredContractByName {
        name: String,
        entry_point: String,
        args: Vec<u8>,
    },
    StoredVersionedContractByName {
        name: String,
        version: Option<ContractVersion>, // defaults to highest enabled version
        entry_point: String,
        args: Vec<u8>,
    },
    StoredVersionedContractByHash {
        hash: ContractPackageHash,
        version: Option<ContractVersion>, // defaults to highest enabled version
        entry_point: String,
        args: Vec<u8>,
    },
    Transfer {
        args: Vec<u8>,
    },
}

impl Display for ExecutableDeployItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ExecutableDeployItem::ModuleBytes { module_bytes, args } => {
                write!(f, "execute module bytes {:10}, args {:10}", HexFmt(&module_bytes), HexFmt(&args))
            }
            ExecutableDeployItem::StoredContractByHash { hash, entry_point, args } => {
                write!(f, "execute stored contract by hash {}, entry point {}, args {:10}", HexFmt(&hash), entry_point, HexFmt(&args))
            }
            ExecutableDeployItem::StoredContractByName { name, entry_point, args } => {
                write!(f, "execute stored contract by name {}, entry point {}, args {:10}", name, entry_point, HexFmt(&args))
            }
            ExecutableDeployItem::StoredVersionedContractByName { name, version: Some(ver), entry_point, args } => {
                write!(f, "execute stored versioned contract {}, version {}, entry point {}, args {:10}", name, ver, entry_point, HexFmt(&args))
            }
            ExecutableDeployItem::StoredVersionedContractByName { name, version: None, entry_point, args } => {
                write!(f, "execute stored versioned contract {}, latest version, entry point {}, args {:10}", name, entry_point, HexFmt(&args))
            }
            ExecutableDeployItem::StoredVersionedContractByHash { hash, version: Some(ver), entry_point, args } => {
                write!(f, "execute stored versioned contract by hash {}, version {}, entry point {}, args {:10}", HexFmt(&hash), ver, entry_point, HexFmt(&args))
            }
            ExecutableDeployItem::StoredVersionedContractByHash { hash, version: None, entry_point, args } => {
                write!(f, "execute stored versioned contract by hash {}, latest version, entry point {}, args {:10}", HexFmt(&hash), entry_point, HexFmt(&args))
            }
            ExecutableDeployItem::Transfer { args } => {
                write!(f, "execute transfer args {:10}", HexFmt(&args))
            }
        }
    }
}

impl Debug for ExecutableDeployItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ExecutableDeployItem::ModuleBytes { module_bytes, args } => f
                .debug_struct("ModuleBytes")
                .field("module_bytes", &format!("[{} bytes]", module_bytes.len()))
                .field("args", &hex::encode(&args))
                .finish(),
            ExecutableDeployItem::StoredContractByHash {
                hash,
                entry_point,
                args,
            } => f
                .debug_struct("StoredContractByHash")
                .field("hash", &HexFmt(&hash))
                .field("entry_point", &entry_point)
                .field("args", &HexFmt(&args))
                .finish(),
            ExecutableDeployItem::StoredContractByName {
                name,
                entry_point,
                args,
            } => f
                .debug_struct("StoredContractByName")
                .field("name", &name)
                .field("entry_point", &entry_point)
                .field("args", &HexFmt(&args))
                .finish(),
            ExecutableDeployItem::StoredVersionedContractByName {
                name,
                version,
                entry_point,
                args,
            } => f
                .debug_struct("StoredVersionedContractByName")
                .field("name", &name)
                .field("version", version)
                .field("entry_point", &entry_point)
                .field("args", &HexFmt(&args))
                .finish(),
            ExecutableDeployItem::StoredVersionedContractByHash {
                hash,
                version,
                entry_point,
                args,
            } => f
                .debug_struct("StoredVersionedContractByHash")
                .field("hash", &HexFmt(&hash))
                .field("version", version)
                .field("entry_point", &entry_point)
                .field("args", &HexFmt(&args))
                .finish(),
            ExecutableDeployItem::Transfer { args } => f
                .debug_struct("Transfer")
                .field("args", &HexFmt(&args))
                .finish(),
        }
    }
}

impl ExecutableDeployItem {
    pub(crate) fn to_contract_hash_key(
        &self,
        account: &Account,
    ) -> Result<Option<Key>, error::Error> {
        match self {
            ExecutableDeployItem::StoredContractByHash { hash, .. }
            | ExecutableDeployItem::StoredVersionedContractByHash { hash, .. } => {
                Ok(Some(Key::from(*hash)))
            }
            ExecutableDeployItem::StoredContractByName { name, .. }
            | ExecutableDeployItem::StoredVersionedContractByName { name, .. } => {
                let key = account.named_keys().get(name).cloned().ok_or_else(|| {
                    error::Error::Exec(execution::Error::NamedKeyNotFound(name.to_string()))
                })?;
                Ok(Some(key))
            }
            ExecutableDeployItem::ModuleBytes { .. } | ExecutableDeployItem::Transfer { .. } => {
                Ok(None)
            }
        }
    }

    pub fn into_runtime_args(self) -> Result<RuntimeArgs, bytesrepr::Error> {
        match self {
            ExecutableDeployItem::ModuleBytes { args, .. }
            | ExecutableDeployItem::StoredContractByHash { args, .. }
            | ExecutableDeployItem::StoredContractByName { args, .. }
            | ExecutableDeployItem::StoredVersionedContractByHash { args, .. }
            | ExecutableDeployItem::StoredVersionedContractByName { args, .. }
            | ExecutableDeployItem::Transfer { args } => {
                let runtime_args: RuntimeArgs = bytesrepr::deserialize(args)?;
                Ok(runtime_args)
            }
        }
    }

    pub fn entry_point_name(&self) -> &str {
        match self {
            ExecutableDeployItem::ModuleBytes { .. } | ExecutableDeployItem::Transfer { .. } => {
                DEFAULT_ENTRY_POINT_NAME
            }
            ExecutableDeployItem::StoredVersionedContractByName { entry_point, .. }
            | ExecutableDeployItem::StoredVersionedContractByHash { entry_point, .. }
            | ExecutableDeployItem::StoredContractByHash { entry_point, .. }
            | ExecutableDeployItem::StoredContractByName { entry_point, .. } => &entry_point,
        }
    }
}
