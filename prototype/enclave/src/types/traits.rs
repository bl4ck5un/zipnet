use crate::interface::DcMessage;
use std::prelude::v1::*;

// various functions for computing a.xor(b)
pub trait Xor {
    // xor returns xor(self, other)
    fn xor(&self, other: &Self) -> Self;
    // xor_mut computes and sets self = xor(self, other)
    fn xor_mut(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = self.xor(other);
    }
}

impl Xor for DcMessage {
    fn xor(&self, other: &Self) -> Self {
        let mut result = DcMessage::zero();
        for i in 0..DC_NET_MESSAGE_LENGTH {
            result.0[i] = self.0[i] ^ other.0[i];
        }

        result
    }
}

// return a reasonable zero value
pub trait Zero {
    fn zero() -> Self;
}

use interface::DC_NET_MESSAGE_LENGTH;

impl Zero for DcMessage {
    fn zero() -> Self {
        DcMessage([0 as u8; DC_NET_MESSAGE_LENGTH])
    }
}

pub trait Sealable {
    fn seal(&self) -> SgxResult<Vec<u8>>;
}

use serde::Serialize;
use sgx_types::SgxResult;
use utils::ser_and_seal_to_vec;

impl<T> Sealable for T
where
    T: Serialize,
{
    fn seal(&self) -> SgxResult<Vec<u8>> {
        ser_and_seal_to_vec(self, b"")
    }
}

pub trait UnsealableAs<T> {
    fn unseal(&self) -> SgxResult<T>;
}

pub trait UnmarshallableAs<T> {
    fn unmarshal(&self) -> SgxResult<T>;
}

pub trait MarshallAs<T> {
    fn marshal(&self) -> SgxResult<T>;
}

pub trait SealAs<T> {
    fn seal(&self) -> SgxResult<T>;
}
