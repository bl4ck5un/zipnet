use std::prelude::v1::*;

use crate::{footprint_sched::SealedFootprintTicket, key::*, params::*, signature::*};

use sgx_types::SGX_HMAC256_KEY_SIZE;
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
#[derive(Copy, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EntityId([u8; USER_ID_LENGTH]);

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

// secret shared by server & user
#[cfg_attr(feature = "trusted", serde(crate = "serde_sgx"))]
#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct ServerSecret {
    pub secret: [u8; SGX_HMAC256_KEY_SIZE],
    // sgx_cmac_128bit_key_t
    pubkey: PubKey,
    sig: Signature,
}

/// Enclave-generated secrets shared with a set of anytrust servers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SealedServerSecrets(pub Vec<u8>);

impl ServerSecret {
    pub fn gen_test(byte: u8) -> Self {
        return ServerSecret {
            secret: [byte; SGX_HMAC256_KEY_SIZE],
            pubkey: PubKey::default(), // dummy values
            sig: Signature::default(), // dummy values
        };
    }
}

#[cfg_attr(feature = "trusted", serde(crate = "serde_sgx"))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSubmissionReq {
    pub user_id: EntityId,
    pub round: u32,
    pub msg: DcMessage,
    pub ticket: SealedFootprintTicket,
    /// When unsealed, this must have the form (H(kpk_1, ..., kpk_ℓ), s_1, ..., s_ℓ) so that the
    /// shared secrets are linked to the relevant servers
    pub shared_secrets: SealedServerSecrets,
}

#[cfg_attr(feature = "trusted", serde(crate = "serde_sgx"))]
#[derive(Debug, Serialize, Deserialize)]
pub struct SignedUserMessage {
    pub user_id: EntityId,
    pub round: u32,
    pub msg: DcMessage,
    pub tee_sig: Signature,
    pub tee_pk: PubKey,
}
