//! Common types used across multiple components.

pub(crate) mod appendable_block;
mod block;
pub mod chainspec;
mod deploy;
mod exit_code;
mod item;
pub mod json_compatibility;
mod node_config;
mod node_id;
mod peers_map;
mod status_feed;
mod timestamp;

use std::{fmt::Display, ops::Deref};

use rand::{CryptoRng, RngCore};
#[cfg(not(test))]
use rand_chacha::ChaCha20Rng;

pub use block::{
    json_compatibility::JsonBlock, Block, BlockBody, BlockHash, BlockHeader, BlockSignatures,
    BlockValidationError, FinalitySignature,
};
pub(crate) use block::{BlockByHeight, BlockHeaderWithMetadata, BlockPayload, FinalizedBlock};
pub(crate) use chainspec::ActivationPoint;
pub use chainspec::Chainspec;
pub use datasize::DataSize;
pub use deploy::{
    Approval, Deploy, DeployHash, DeployHeader, DeployMetadata, DeployOrTransferHash,
    DeployValidationFailure, Error as DeployError, ExcessiveSizeError as ExcessiveSizeDeployError,
};
pub use exit_code::ExitCode;
pub use item::{Item, Tag};
pub use node_config::NodeConfig;
pub(crate) use node_id::NodeId;
pub use peers_map::PeersMap;
use serde::{Deserialize, Serialize};
pub use status_feed::{ChainspecInfo, GetStatusResult, StatusFeed};
pub use timestamp::{TimeDiff, Timestamp};

/// An object-safe RNG trait that requires a cryptographically strong random number generator.
pub trait CryptoRngCore: CryptoRng + RngCore {}

impl<T> CryptoRngCore for T where T: CryptoRng + RngCore + ?Sized {}

/// The cryptographically secure RNG used throughout the node.
#[cfg(not(test))]
pub type NodeRng = ChaCha20Rng;

/// The RNG used throughout the node for testing.
#[cfg(test)]
pub type NodeRng = crate::testing::TestRng;

/// An in-memory object that can possibly be shared with user parts of the system.
///
/// In general, this should only be used for immutable, content-addressed objects.
///
/// This type exists solely to switch between `Box` and `Arc` based behavior, future updates should
/// deprecate this in favor of using `Arc`s directly or turning `LoadedItem` into a newtype.
#[derive(DataSize, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LoadedItem<T> {
    /// An owned copy of the object.
    Owned(Box<T>),
}

impl<T> Deref for LoadedItem<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            LoadedItem::Owned(obj) => &*obj,
        }
    }
}

impl<T> LoadedItem<T> {
    /// Creates a new owned instance of the object.
    #[inline]
    fn owned_new(inner: T) -> Self {
        LoadedItem::Owned(Box::new(inner))
    }

    /// Converts a loaded object into an instance of `T`.
    ///
    /// May clone the object as a result. This method should not be used in new code, it exists
    /// solely to bridge old interfaces with the `LoadedItem`.
    #[inline]
    pub(crate) fn into_inner(self) -> T {
        match self {
            LoadedItem::Owned(inner) => *inner,
        }
    }
}

impl<T> Display for LoadedItem<T>
where
    T: Display,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadedItem::Owned(inner) => inner.fmt(f),
        }
    }
}

impl<T> Serialize for LoadedItem<T>
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            LoadedItem::Owned(inner) => inner.serialize(serializer),
        }
    }
}

impl<'de, T> Deserialize<'de> for LoadedItem<T>
where
    T: Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer).map(LoadedItem::owned_new)
    }
}
