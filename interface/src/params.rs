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

/// There are these many rounds per window
pub const DC_NET_ROUNDS_PER_WINDOW: u32 = 100;
/// A user is allowed to talk this many times per window
pub const DC_NET_MSGS_PER_WINDOW: u32 = 10;

/// The size of an anytrust shared secret
pub const SERVER_KEY_LENGTH: usize = DC_NET_MESSAGE_LENGTH;

/// The size of a sealed secret key. Although the secret key is only 32-byte, the sealed version is
/// quite large and we can't go much smaller than 1024.
pub const SEALED_SGX_SIGNING_KEY_LENGTH: usize = 1024;

#[cfg_attr(feature = "trusted", serde(crate = "serde_sgx"))]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Ord, PartialOrd, Serialize, Deserialize)]
pub struct RoundInfo {
    pub round: u32,
    pub window: u32,
}

impl RoundInfo {
    // Return the next round, respecting window size
    pub fn next_round(&self) -> RoundInfo {
        let (new_round, new_window) = if self.round == DC_NET_ROUNDS_PER_WINDOW - 1 {
            (0, self.window + 1)
        } else {
            (self.round + 1, self.window)
        };

        RoundInfo {
            round: new_round,
            window: new_window,
        }
    }

    // Return the previous round, respecting window size
    pub fn prev_round(&self) -> Option<RoundInfo> {
        match (self.round, self.window) {
            (0, 0) => None,
            (0, w) => Some(RoundInfo {
                round: DC_NET_ROUNDS_PER_WINDOW - 1,
                window: w - 1,
            }),
            (r, w) => Some(RoundInfo {
                round: r - 1,
                window: w,
            }),
        }
    }

    /// Return whether this is the first round of the first window
    pub fn is_zero(&self) -> bool {
        self.round == 0 && self.window == 0
    }
}
