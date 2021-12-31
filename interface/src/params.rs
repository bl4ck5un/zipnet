/// User id is sha-256 hash of some public key
pub const USER_ID_LENGTH: usize = 32;
pub const USER_ID_MAX_LEN: usize = 32;

/// The num of bits in a footprint. must be smaller than 32 (checked in enclave)
pub const FOOTPRINT_BIT_SIZE: usize = 3;

/// The number of scheduling slots. This should be larger than DC_NET_N_SLOTS to avoid collision.
pub const FOOTPRINT_N_SLOTS: usize = DC_NET_N_SLOTS * 4;

/// The number of slots in a DC net message
pub const DC_NET_N_SLOTS: usize = 128;
/// The number of bytes in each DC net slot
pub const DC_NET_MESSAGE_LENGTH: usize = 256;

/// The size of an anytrust shared secret
pub const SERVER_KEY_LENGTH: usize = DC_NET_MESSAGE_LENGTH;

/// The size of a sealed secret key. Although the secret key is only 32-byte, the sealed version is
/// quite large and we can't go much smaller than 1024.
pub const SEALED_SGX_SIGNING_KEY_LENGTH: usize = 1024;

#[cfg_attr(feature = "trusted", serde(crate = "serde_sgx"))]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct RoundInfo {
    pub round: u32,
    pub window: u32,
    pub msgs_per_window: u32,
    pub rounds_per_window: u32,
}

impl RoundInfo {
    // Increment the round counter, respecting window size
    pub fn incr_round(&self) -> RoundInfo {
        let (new_round, new_window) = if self.round == self.rounds_per_window - 1 {
            (0, self.window + 1)
        } else {
            (self.round + 1, self.window)
        };

        RoundInfo {
            round: new_round,
            window: new_window,
            msgs_per_window: self.msgs_per_window,
            rounds_per_window: self.rounds_per_window,
        }
    }
}
