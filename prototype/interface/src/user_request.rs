use std::collections::BTreeSet;
use std::prelude::v1::*;

use crate::{params::*, sealed_types::*, sgx_protected_keys::*};

use sgx_types::SGX_HMAC256_KEY_SIZE;
use sha2::{Digest, Sha256};
use std::fmt::{Debug, Formatter, Result as FmtResult};

big_array! { BigArray; }

// a wrapper around RawMessage so that we can impl traits
#[cfg_attr(feature = "trusted", serde(crate = "serde_sgx"))]
#[derive(Clone, Serialize, Deserialize)]
pub struct DcMessage(#[serde(with = "BigArray")] pub [u8; DC_NET_MESSAGE_LENGTH]);

impl Default for DcMessage {
    fn default() -> DcMessage {
        DcMessage([0u8; DC_NET_MESSAGE_LENGTH])
    }
}

#[cfg(feature = "trusted")]
use sgx_rand::{Rand, Rng};

#[cfg(feature = "trusted")]
impl Rand for DcMessage {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        let mut r = DcMessage::default();
        rng.fill_bytes(&mut r.0);

        r
    }
}

impl std::cmp::PartialEq for DcMessage {
    fn eq(&self, other: &Self) -> bool {
        self.0.iter().zip(&other.0).all(|(x, y)| x == y)
    }
}

impl Debug for DcMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&hex::encode(&self))
    }
}

impl From<[u8; DC_NET_MESSAGE_LENGTH]> for DcMessage {
    fn from(raw: [u8; DC_NET_MESSAGE_LENGTH]) -> Self {
        DcMessage(raw)
    }
}

impl AsRef<[u8]> for DcMessage {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8; DC_NET_MESSAGE_LENGTH]> for DcMessage {
    fn as_ref(&self) -> &[u8; DC_NET_MESSAGE_LENGTH] {
        &self.0
    }
}

#[cfg_attr(feature = "trusted", serde(crate = "serde_sgx"))]
#[derive(Copy, Clone, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(pub [u8; USER_ID_LENGTH]);

impl Debug for EntityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&hex::encode(self.0))
    }
}

impl AsRef<[u8]> for EntityId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; USER_ID_LENGTH]> for EntityId {
    fn from(raw: [u8; USER_ID_LENGTH]) -> Self {
        EntityId(raw)
    }
}

/// Computes a group ID given a list of entity IDs
pub fn compute_group_id(ids: &BTreeSet<EntityId>) -> EntityId {
    // The group ID of a set of entities is the hash of their IDs, concatenated in ascending order.
    // There's also the context str of "grp" prepended.
    let mut hasher = Sha256::new();
    hasher.update(b"grp");
    for id in ids {
        hasher.update(&id.0);
    }
    let digest = hasher.finalize();

    let mut id = EntityId::default();
    id.0.copy_from_slice(&digest);
    id
}

pub fn compute_anytrust_group_id(keys: &Vec<KemPubKey>) -> EntityId {
    let mut entity_ids: Vec<EntityId> =
        keys.iter().map(|k: &KemPubKey| k.get_entity_id()).collect();

    // TODO: test if this does the thing
    entity_ids.sort();

    // hash everything
    let mut hasher = Sha256::new();
    hasher.update("dcnetid");
    for eid in entity_ids.iter() {
        hasher.update(eid.0);
    }

    let digest = hasher.finalize();

    let mut id = EntityId::default();
    id.0.copy_from_slice(&digest);

    id
}

#[cfg_attr(feature = "trusted", serde(crate = "serde_sgx"))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSubmissionReq {
    pub user_id: EntityId,
    pub anytrust_group_id: EntityId,
    pub round: u32,
    pub msg: DcMessage,
    pub ticket: SealedFootprintTicket,
    /// When unsealed, this must have the form (H(kpk_1, ..., kpk_ℓ), s_1, ..., s_ℓ) so that the
    /// shared secrets are linked to the relevant servers
    pub shared_secrets: SealedServerSecrets,
}
