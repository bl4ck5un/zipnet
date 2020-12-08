#![no_std]

extern crate cfg_if;
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "untrusted")] {
        #[macro_use]
        extern crate serde;
        #[macro_use]
        extern crate serde_big_array;
        extern crate std;
    } else if #[cfg(feature = "trusted")] {
        #[macro_use]
        extern crate serde_sgx;
        #[macro_use]
        extern crate serde_big_array_sgx;
        extern crate sgx_tstd as std;
        extern crate sha2;
        extern crate byteorder;
        extern crate sgx_rand;
        #[macro_use]
        extern crate sgx_rand_derive;
    } else {
        compile_error!{"must be either trusted or untrusted"}
    }
}

use std::prelude::v1::*;

big_array! { BigArray; }

mod footprint_sched;
mod key;
mod message;
mod params;

pub use footprint_sched::*;
pub use key::*;
pub use message::*;
pub use params::*;
