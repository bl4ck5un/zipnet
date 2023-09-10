use crypto::{
    SgxPrivateKey,
    SharedSecretsDbClient,
};
use interface::*;
use messages_types;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_cbor;
use sgx_tseal::SgxSealedData;
use sgx_types::sgx_status_t::{SGX_ERROR_INVALID_PARAMETER, SGX_ERROR_UNEXPECTED};
use sgx_types::{sgx_sealed_data_t, SgxResult};
use std::borrow::ToOwned;
use std::vec::Vec;

fn serialize_to_vec<T: Serialize>(v: &T) -> SgxResult<Vec<u8>> {
    serde_cbor::to_vec(v).map_err(|e| {
        println!("can't serialize_to_vec {}", e);
        SGX_ERROR_UNEXPECTED
    })
}

fn deserialize_from_vec<T: DeserializeOwned>(bin: &[u8]) -> SgxResult<T> {
    serde_cbor::from_slice::<T>(bin).map_err(|e| {
        println!("can't deserialize_from_vec {}", e);
        SGX_ERROR_INVALID_PARAMETER
    })
}

fn ser_and_seal_to_vec<T: Serialize>(a: &T, ad: &[u8]) -> SgxResult<Vec<u8>> {
    let bin = match serde_cbor::ser::to_vec(a) {
        Ok(b) => b,
        Err(e) => {
            println!("can't serialize {}", e);
            return Err(SGX_ERROR_INVALID_PARAMETER);
        }
    };

    let sealed = SgxSealedData::<[u8]>::seal_data(ad, &bin)?;
    let mut sealed_bin = vec![0u8; (sealed.get_payload_size() + 1024) as usize];
    match unsafe {
        sealed.to_raw_sealed_data_t(
            sealed_bin.as_mut_ptr() as *mut sgx_sealed_data_t,
            sealed_bin.len() as u32,
        )
    } {
        Some(_) => Ok(sealed_bin),
        None => {
            println!("can't seal. cap {}", sealed_bin.len());
            Err(SGX_ERROR_INVALID_PARAMETER)
        }
    }
}

// TODO: make input generic AsRef<[u8]>
/// Unseal bytes and unmarshal to a T. Returns (T, additional data)
fn unseal_vec_and_deser<T: DeserializeOwned + Default>(input: &Vec<u8>) -> SgxResult<(T, Vec<u8>)> {
    let mut bin = input.clone();

    let sealed_data = unsafe {
        match SgxSealedData::<[u8]>::from_raw_sealed_data_t(
            bin.as_mut_ptr() as *mut sgx_sealed_data_t,
            bin.len() as u32,
        ) {
            Some(t) => t,
            None => {
                return Err(SGX_ERROR_INVALID_PARAMETER);
            }
        }
    };

    let unsealed = sealed_data.unseal_data()?;
    let unsealed_slice = unsealed.get_decrypt_txt();

    // unmarshal
    let t = match serde_cbor::de::from_slice(unsealed_slice) {
        Ok(t) => t,
        Err(_e) => {
            return Err(SGX_ERROR_INVALID_PARAMETER);
        }
    };

    Ok((t, unsealed.get_additional_txt().to_vec()))
}

/// a few useful traits
/// This is a private trait
///
/// Other code should use SealInto* and UnsealInto*
trait Sealable {
    fn seal(&self, ad: Option<&[u8]>) -> SgxResult<Vec<u8>>;
}

/// Any serializable type can be sealed
impl<T> Sealable for T
where
    T: Serialize,
{
    fn seal(&self, ad: Option<&[u8]>) -> SgxResult<Vec<u8>> {
        ser_and_seal_to_vec(
            self,
            match ad {
                Some(ad) => ad,
                None => b"",
            },
        )
    }
}

/// Seal and store sealed bytes in type T
pub trait SealInto<T> {
    fn seal_into(&self) -> SgxResult<T>;
}

pub trait UnsealableInto<T> {
    fn unseal_into(&self) -> SgxResult<T>;
}

impl SealInto<SealedSigPrivKey> for NoSgxPrivateKey {
    fn seal_into(&self) -> SgxResult<SealedSigPrivKey> {
        Ok(SealedSigPrivKey(self.seal(None)?))
    }
}

impl UnsealableInto<NoSgxPrivateKey> for SealedSigPrivKey {
    fn unseal_into(&self) -> sgx_types::SgxResult<NoSgxPrivateKey> {
        Ok(unseal_vec_and_deser(&self.0)?.0) // ignore the ad
    }
}

impl  SealInto<SealedSharedSecretsDbClient> for SharedSecretsDbClient {
    fn seal_into(&self) -> SgxResult<SealedSharedSecretsDbClient> {
        let mut sealed_shared_secrets = SealedSharedSecretsDbClient::default();
        sealed_shared_secrets.round = self.round;

        for (k, s) in self.db.iter() {
            // authenticate public keys and rounds in "ad"
            let mut ad = Vec::new();
            ad.extend_from_slice(&k.0);
            ad.extend_from_slice(&self.round.to_ne_bytes());

            sealed_shared_secrets
                .db
                .insert(k.to_owned(), s.seal(Some(&ad))?);
        }
        
        Ok(sealed_shared_secrets)
    }
}

impl UnsealableInto<SharedSecretsDbClient> for SealedSharedSecretsDbClient {
    fn unseal_into(&self) -> sgx_types::SgxResult<SharedSecretsDbClient> {
        let mut db = SharedSecretsDbClient::default();
        db.round = self.round;
        for (k, v) in self.db.iter() {
            // exoected ad = pk || round
            let mut expected_ad = Vec::new();
            expected_ad.extend_from_slice(&k.0);
            expected_ad.extend_from_slice(&self.round.to_ne_bytes());

            let (secret, ad) = unseal_vec_and_deser(&v)?;

            // check that exptected_ad == ad
            if expected_ad != ad {
                error!("unseal SharedSecretsDbClient failed. Ad not matching");
                return Err(SGX_ERROR_INVALID_PARAMETER);
            }

            db.db.insert(k.to_owned(), secret);
        }
        
        Ok(db)
    }
}