use core::convert::TryFrom;
use crypto::SgxPrivateKey;
use interface::*;
use sgx_rand::Rng;
use sgx_types::sgx_status_t::SGX_ERROR_UNEXPECTED;
use sgx_types::SgxResult;
use std::string::ToString;
use std::vec;

pub fn new_sgx_keypair_ext_internal(role: &str) -> SgxResult<(SgxPrivateKey, AttestedPublicKey)> {
    let mut rand = sgx_rand::SgxRng::new().map_err(|e| {
        error!("cant create rand {}", e);
        SGX_ERROR_UNEXPECTED
    })?;

    // generate a random secret key
    let sk = rand.gen::<SgxPrivateKey>();
    let attested_key = AttestedPublicKey {
        pk: SgxSigningPubKey::try_from(&sk)?,
        role: role.to_string(),
        tee_linkable_attestation: vec![0], // TODO: add attestation
    };

    Ok((sk, attested_key))
}

pub fn new_keypair_ext_internal(role: &str) -> SgxResult<(NoSgxPrivateKey, AttestedPublicKeyNoSGX)> {
    let mut rand = sgx_rand::SgxRng::new().map_err(|e| {
        error!("can't create rand {}", e);
        SGX_ERROR_UNEXPECTED
    })?;

    // generate a random secret key
    let sk = rand.gen::<NoSgxPrivateKey>();
    let attested_key = AttestedPublicKeyNoSGX {
        pk: NoSgxProtectedKeyPub::try_from(&sk).map_err(|e| {
            error!("can't generate NoSgxProtectedKeyPub from NoSgxPrivateKey: {}", e);
            SGX_ERROR_UNEXPECTED
        })?,
        role: role.to_string(),
        tee_linkable_attestation: vec![0], // TODO: add attestation
    };

    Ok((sk, attested_key))
}
