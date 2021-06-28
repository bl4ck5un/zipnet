use sgx_types;
use sgx_urts;

use sgx_status_t::SGX_SUCCESS;
use sgx_types::*;
use sgx_urts::SgxEnclave;
use std::path::PathBuf;

use interface::*;

// error type for enclave operations
use sgx_types::sgx_status_t;
use std::error::Error;
use std::fmt::{Display, Formatter};

use quick_error::quick_error;
use rand::Rng;
use serde;
use sgx_types::sgx_device_status_t::SGX_DISABLED_UNSUPPORTED_CPU;
use sgx_types::sgx_status_t::{SGX_ERROR_INVALID_PARAMETER, SGX_ERROR_UNEXPECTED};

quick_error! {
    #[derive(Debug)]
    pub enum EnclaveError {
        SgxError(err: sgx_status_t) {
            from(sgx_status_t)
        }
        EnclaveLogicError(err: sgx_status_t) {
            from(sgx_status_t)
        }
        MarshallError(e: serde_cbor::Error) {
            from(e: serde_cbor::Error) -> (e)
        }
    }
}

pub type EnclaveResult<T> = Result<T, EnclaveError>;

type GenericEcallFn = unsafe extern "C" fn(
    eid: sgx_enclave_id_t,
    retval: *mut sgx_status_t,
    inp: *const u8,
    inp_len: usize,
    output: *mut u8,
    output_size: usize,
    output_used: *mut usize,
) -> sgx_status_t;

// E calls
extern "C" {
    fn ecall_entrypoint(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        ecall_id_raw: u8,
        inp: *const u8,
        inp_len: usize,
        output: *mut u8,
        output_cap: usize,
        output_used: *mut usize,
    ) -> sgx_status_t;

    fn ecall_aggregate(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        new_input_ptr: *const u8,
        new_input_len: usize,
        current_aggregation_ptr: *const u8,
        current_aggregation_len: usize,
        sealed_tee_prv_ptr: *const u8,
        sealed_tee_prv_len: usize,
        output_aggregation_ptr: *mut u8,
        output_size: usize,
        output_bytes_written: *mut usize,
    ) -> sgx_status_t;

    fn test_main_entrance(eid: sgx_enclave_id_t, retval: *mut sgx_status_t) -> sgx_status_t;
}

const ENCLAVE_OUTPUT_BUF_SIZE: usize = 1024000;

#[derive(Debug)]
pub struct DcNetEnclave {
    enclave: sgx_urts::SgxEnclave,
    ecall_out_buf: Vec<u8>,
}

impl DcNetEnclave {
    pub fn init(enclave_file: &'static str) -> EnclaveResult<Self> {
        let enclave_path = PathBuf::from(enclave_file);

        let mut launch_token: sgx_launch_token_t = [0; 1024];
        let mut launch_token_updated: i32 = 0;
        // call sgx_create_enclave to initialize an enclave instance
        // Debug Support: set 2nd parameter to 1
        let debug = 1;
        let mut misc_attr = sgx_misc_attribute_t {
            secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
            misc_select: 0,
        };

        let enclave = SgxEnclave::create(
            enclave_path,
            debug,
            &mut launch_token,
            &mut launch_token_updated,
            &mut misc_attr,
        )
        .map_err(EnclaveError::SgxError)?;

        Ok(Self {
            enclave: enclave,
            ecall_out_buf: vec![0; ENCLAVE_OUTPUT_BUF_SIZE],
        })
    }

    pub fn destroy(self) {
        self.enclave.destroy();
    }

    pub fn get_eid(&self) -> sgx_types::sgx_enclave_id_t {
        self.enclave.geteid()
    }

    fn make_generic_ecall<I, O>(&mut self, ecall_id: EcallId, inp: &I) -> EnclaveResult<O>
    where
        I: serde::Serialize,
        O: serde::de::DeserializeOwned,
    {
        let marshaled_input = serde_cbor::to_vec(&inp)?;

        println!("marshalled inp len {}", marshaled_input.len());
        println!("outbuf len {}", self.ecall_out_buf.len());

        let mut ret = SGX_SUCCESS;
        let mut outbuf_used = 0usize;

        // Call FFI
        let call_ret = unsafe {
            ecall_entrypoint(
                self.enclave.geteid(),
                &mut ret,
                ecall_id as u8,
                marshaled_input.as_ptr(),
                marshaled_input.len(),
                self.ecall_out_buf.as_mut_ptr(),
                self.ecall_out_buf.len(),
                &mut outbuf_used,
            )
        };

        // Check for errors
        if call_ret != SGX_SUCCESS {
            return Err(EnclaveError::SgxError(call_ret));
        }
        if ret != SGX_SUCCESS {
            return Err(EnclaveError::SgxError(ret));
        }

        println!("ecall succeed. buf used {}", outbuf_used);

        let output: O =
            serde_cbor::from_slice(&self.ecall_out_buf[..outbuf_used]).map_err(|e| {
                println!("can't unmarshal: {}", e);
                EnclaveError::MarshallError(e)
            })?;

        Ok(output)
    }

    /// new_sgx_protected_key creates a new key pair on P-256 and returns the sealed secret key.
    /// This method can be used for creating signing keys and KEM private keys.
    /// Use unseal_to_pubkey to unseal the key and compute its public key.
    pub fn new_sgx_protected_key(&mut self, role: String) -> EnclaveResult<SealedKey> {
        let output: SealedKey = self.make_generic_ecall(EcallId::EcallNewSgxKeypair, &role)?;
        println!("output {:?}", output);
        Ok(output)
    }

    // unseal the key to see its public key
    pub fn unseal_to_public_key_on_p256(
        &mut self,
        sealed_private_key: &SealedKey,
    ) -> EnclaveResult<SgxProtectedKeyPub> {
        let output: SgxProtectedKeyPub =
            self.make_generic_ecall(EcallId::EcallUnsealToPublicKey, &sealed_private_key)?;
        println!("output {:?}", output);
        Ok(output)
    }

    /// Given a message and the relevant scheduling ticket, constructs a round message for sending
    /// to an aggregator
    pub fn user_submit_round_msg(
        &mut self,
        submission_req: &UserSubmissionReq,
        sealed_usk: &SealedKey,
    ) -> EnclaveResult<MarshalledSignedUserMessage> {
        let marshaled_signed_msg: Vec<u8> =
            self.make_generic_ecall(EcallId::EcallUserSubmit, &(submission_req, sealed_usk))?;
        Ok(MarshalledSignedUserMessage(marshaled_signed_msg))
    }

    /// Makes an empty aggregation state for the given round and wrt the given anytrust nodes
    pub fn new_aggregate(
        &self,
        round: u32,
        anytrust_group_id: &EntityId,
    ) -> EnclaveResult<MarshalledPartialAggregate> {
        // add_to_aggregate will create a new_aggregate when given an empty blob
        Ok(MarshalledPartialAggregate(Vec::new()))
    }

    /// Adds the given input from a user to the given partial aggregate
    /// Note: if marshalled_current_aggregation is empty (len = 0), an empty aggregation is created
    //  and the signed message is aggregated into that.
    pub fn add_to_aggregate(
        &mut self,
        agg: &mut MarshalledPartialAggregate,
        new_input: &MarshalledSignedUserMessage,
        sealed_tee_signing_key: &SealedKey,
    ) -> EnclaveResult<()> {
        let new_agg: MarshalledPartialAggregate = self.make_generic_ecall(
            EcallId::EcallAddToAggregate,
            &(new_input, agg.clone(), sealed_tee_signing_key),
        )?;

        agg.0.clear();
        agg.0.extend_from_slice(&new_agg.0);

        Ok(())
    }

    /// Constructs an aggregate message from the given state. The returned blob is to be sent to
    /// the parent aggregator or an anytrust server.
    /// TODO: 1) what is this supposed to achieve? i.e., no why just send partial aggregate to the any trust server?
    /// TODO: 2) should AggregateBlob contain all of the user ids? If so, AggregateBlob is also the result of user_submit which contains only one user id.
    pub fn finalize_aggregate(
        &self,
        agg: &MarshalledPartialAggregate,
    ) -> EnclaveResult<MarshalledSignedUserMessage> {
        unimplemented!()
    }

    pub fn run_enclave_tests(&self) -> SgxError {
        let mut retval = SGX_SUCCESS;
        unsafe {
            test_main_entrance(self.enclave.geteid(), &mut retval);
        }
        if retval != SGX_SUCCESS {
            return Err(retval);
        }
        Ok(())
    }

    /// Create a new TEE protected secret key. Derives shared secrets with all the given KEM pubkeys.
    /// Returns UserRegistration that contains sealed secrets, a sealed private key, and attestation
    /// information to send to anytrust nodes.
    pub fn register_user(
        &mut self,
        pubkeys: &[KemPubKey],
    ) -> EnclaveResult<(
        SealedServerSecrets,
        SealedKey,
        EntityId,
        UserRegistrationBlob,
    )> {
        let output: UserRegistration =
            self.make_generic_ecall(EcallId::EcallRegisterUser, &pubkeys.to_vec())?;
        println!("output {:?}", output);
        Ok((
            output.get_sealed_server_secrets().to_owned(),
            output.get_sealed_usk().to_owned(),
            output.get_user_id(),
            UserRegistrationBlob(output.get_registration_proof().to_vec()),
        ))
    }

    /// Create a new TEE protected secret key. Returns AggregatorRegistration that contains the sealed private key and attestation information to send to anytrust nodes.
    pub fn register_aggregator(
        &mut self,
    ) -> EnclaveResult<(SealedKey, EntityId, AggRegistrationBlob)> {
        let sealed_sk: SealedKey =
            self.make_generic_ecall(EcallId::EcallNewSgxKeypair, &"agg".to_string())?;
        Ok((
            sealed_sk.clone(),
            EntityId::from(&sealed_sk.pk),
            AggRegistrationBlob(sealed_sk.tee_linkable_attestation),
        ))
    }

    // TODO: Write anytrust node function that receives registration blobs and processes them
}

#[cfg(test)]
mod enclave_tests {
    const TEST_ENCLAVE_PATH: &'static str = "/sgxdcnet/lib/enclave.signed.so";

    use super::DcNetEnclave;

    extern crate base64;
    extern crate hex;
    extern crate hexdump;
    extern crate interface;
    extern crate sgx_types;

    use hex::FromHex;
    use interface::{
        DcMessage, EntityId, SealedFootprintTicket, SealedKey, SealedServerSecrets,
        SgxProtectedKeyPub, UserSubmissionReq, DC_NET_MESSAGE_LENGTH,
        SEALED_SGX_SIGNING_KEY_LENGTH, USER_ID_LENGTH,
    };
    use sgx_types::SGX_ECP256_KEY_SIZE;

    #[test]
    fn key_seal_unseal() {
        let mut enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();
        let sealed = enc.new_sgx_protected_key("test".to_string()).unwrap();

        println!("hererererer");
        let pk_unsealed = enc.unseal_to_public_key_on_p256(&sealed).unwrap();

        assert_eq!(pk_unsealed, sealed.pk);
    }

    #[test]
    fn user_submit_round_msg() {
        let mut enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();

        // create server public keys
        let spks = create_server_pubkeys(&mut enc, 10);
        let (user_reg_server_secrets, user_reg_sealed_key, user_reg_uid, user_reg_proof) =
            enc.register_user(&spks).unwrap();

        let req_1 = UserSubmissionReq {
            user_id: user_reg_uid,
            anytrust_group_id: user_reg_server_secrets.anytrust_group_id,
            round: 0u32,
            msg: DcMessage([0u8; DC_NET_MESSAGE_LENGTH]),
            ticket: SealedFootprintTicket(vec![0; 1]),
            server_secrets: user_reg_server_secrets,
        };
        let resp_1 = enc
            .user_submit_round_msg(&req_1, &user_reg_sealed_key)
            .unwrap();

        enc.destroy();
    }

    fn create_server_pubkeys(
        enc: &mut DcNetEnclave,
        num_of_servers: i32,
    ) -> Vec<SgxProtectedKeyPub> {
        let mut pks = Vec::new();
        for i in 0..num_of_servers {
            let sk = enc.new_sgx_protected_key("test".to_string()).expect("key");
            pks.push(sk.pk);
        }

        pks
    }

    #[test]
    fn aggregation() {
        let mut enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();

        // create server public keys
        let spks = create_server_pubkeys(&mut enc, 10);

        // create a fake user
        let (user_reg_server_secrets, user_reg_sealed_key, user_reg_uid, user_reg_proof) =
            enc.register_user(&spks).unwrap();

        let req_1 = UserSubmissionReq {
            user_id: user_reg_uid,
            anytrust_group_id: user_reg_server_secrets.anytrust_group_id,
            round: 0u32,
            msg: DcMessage([0u8; DC_NET_MESSAGE_LENGTH]),
            ticket: SealedFootprintTicket(vec![0; 1]),
            server_secrets: user_reg_server_secrets,
        };

        let resp_1 = enc
            .user_submit_round_msg(&req_1, &user_reg_sealed_key)
            .unwrap();

        let agg = enc.register_aggregator().expect("agg");

        let mut empty_agg = enc.new_aggregate(0, &EntityId::default()).unwrap();
        enc.add_to_aggregate(&mut empty_agg, &resp_1, &agg.0)
            .unwrap();

        // this should error because user is already in
        assert!(enc
            .add_to_aggregate(&mut empty_agg, &resp_1, &agg.0)
            .is_err());

        let user_2 = enc.register_user(&spks).unwrap();

        let req_2 = UserSubmissionReq {
            user_id: user_2.2,
            anytrust_group_id: user_2.0.anytrust_group_id,
            round: 0u32,
            msg: DcMessage([1u8; DC_NET_MESSAGE_LENGTH]),
            ticket: SealedFootprintTicket(vec![0; 1]),
            server_secrets: user_2.0,
        };
        let resp_2 = enc.user_submit_round_msg(&req_2, &user_2.1).unwrap();

        enc.add_to_aggregate(&mut empty_agg, &resp_2, &agg.0)
            .unwrap();

        enc.destroy();
    }

    use rand;

    #[test]
    fn register_user() {
        let mut enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();

        let mut pks = Vec::new();
        for i in 0..10 {
            let sk = enc.new_sgx_protected_key("user".to_string()).expect("key");
            pks.push(sk.pk);
        }

        let (user_reg_server_secrets, user_reg_sealed_key, user_reg_uid, user_reg_proof) =
            enc.register_user(&pks).unwrap();

        let pk = enc
            .unseal_to_public_key_on_p256(&user_reg_sealed_key)
            .unwrap();
        assert_eq!(EntityId::from(&pk), user_reg_uid);

        enc.destroy();
    }

    #[test]
    fn register_agg() {
        let mut enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();

        let (agg_sealed_key, agg_id, agg_reg_proof) = enc.register_aggregator().unwrap();

        let pk = enc.unseal_to_public_key_on_p256(&agg_sealed_key).unwrap();
        assert_eq!(EntityId::from(&pk), agg_id);

        enc.destroy();
    }

    #[test]
    fn enclave_tests() {
        println!("===begin enclave tests");
        let enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();

        enc.run_enclave_tests().unwrap();

        enc.destroy();
        println!("===end enclave tests");
    }
}
