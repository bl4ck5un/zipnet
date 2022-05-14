use sgx_types;
use sgx_urts;

use sgx_status_t::SGX_SUCCESS;
use sgx_types::*;
use sgx_urts::SgxEnclave;
use std::collections::BTreeSet;
use std::path::PathBuf;

use interface::*;
use std::time::Instant;

// error type for enclave operations
use quick_error::quick_error;
use sgx_types::sgx_status_t;

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

// Ecalls
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

    fn test_main_entrance(eid: sgx_enclave_id_t, retval: *mut sgx_status_t) -> sgx_status_t;
}

const ENCLAVE_OUTPUT_BUF_SIZE: usize = 30 * 1024000; // 5MB buffer should be enough?

#[derive(Clone, Debug)]
pub struct DcNetEnclave {
    enclave: sgx_urts::SgxEnclave,
}

macro_rules! gen_ecall_stub {
    ( $name:expr, $type_i:ty, $type_o: ty, $fn_name: ident) => {
        pub fn $fn_name(
            enclave_id: crate::enclave_wrapper::sgx_enclave_id_t,
            inp: $type_i,
        ) -> crate::EnclaveResult<$type_o> {
            let very_begining = std::time::Instant::now();
            let mut start = std::time::Instant::now();

            let marshaled_input = serde_cbor::to_vec(&inp)?;

            let time_marshal_input = start.elapsed();
            start = std::time::Instant::now();

            let mut ret = crate::enclave_wrapper::SGX_SUCCESS;
            let mut out_buf = vec![0u8; crate::enclave_wrapper::ENCLAVE_OUTPUT_BUF_SIZE];
            let mut outbuf_used = 0usize;

            let time_allocate_out_buf = start.elapsed();
            start = std::time::Instant::now();

            // Call FFI
            let call_ret = unsafe {
                crate::enclave_wrapper::ecall_entrypoint(
                    enclave_id,
                    &mut ret,
                    $name as u8,
                    marshaled_input.as_ptr(),
                    marshaled_input.len(),
                    out_buf.as_mut_ptr(),
                    out_buf.len(),
                    &mut outbuf_used,
                )
            };

            let time_ecall = start.elapsed();
            start = std::time::Instant::now();

            // Check for errors
            if call_ret != crate::enclave_wrapper::sgx_status_t::SGX_SUCCESS {
                return Err(crate::enclave_wrapper::EnclaveError::SgxError(call_ret));
            }
            if ret != crate::enclave_wrapper::sgx_status_t::SGX_SUCCESS {
                return Err(crate::enclave_wrapper::EnclaveError::EnclaveLogicError(ret));
            }

            let output: $type_o = serde_cbor::from_slice(&out_buf[..outbuf_used]).map_err(|e| {
                log::error!("can't unmarshal: {}", e);
                crate::enclave_wrapper::EnclaveError::MarshallError(e)
            })?;

            let time_unmarshal = start.elapsed();
            let total = very_begining.elapsed();

            // print time only if ecall took more than 100ms
            if total > std::time::Duration::from_millis(100) {
                info!(
                    "Ecall {:?} took {:?}. MAR={:?}({}B), ALLOC={:?}, EC={:?}, UNMAR={:?}({}B)",
                    $name,
                    total,
                    time_marshal_input,
                    marshaled_input.len(),
                    time_allocate_out_buf,
                    time_ecall,
                    time_unmarshal,
                    outbuf_used
                );
            }

            Ok(output)
        }
    };
}

/// This macro generates a bunch of functions that will check the ecall is given the right types
macro_rules! match_ecall_ids {
    (
        $(($name:ident, $type_i:ty, $type_o: ty, $fn_name: ident),)+
    ) => {
            $(
                gen_ecall_stub! {$name, $type_i, $type_o, $fn_name}
            )+
        }
}

mod ecall_allowed {
    use interface::*;
    use EcallId::*;

    use std::collections::BTreeSet;

    match_ecall_ids! {
        (
            EcallNewSgxKeypair,
            String,
            (Vec<u8>, AttestedPublicKey),
            new_sgx_keypair
        ),
        (
            EcallUnsealToPublicKey,
            &Vec<u8>,
            SgxProtectedKeyPub,
            unseal_to_public_key),
        (
            EcallNewUser,
            &[ServerPubKeyPackage],
            (SealedSharedSecretDb, SealedSigPrivKey, UserRegistrationBlob),
            new_user
        ),
        (
            EcallNewServer,
            // input
            (),
            // output
            (SealedSigPrivKey, SealedKemPrivKey, ServerRegistrationBlob),
            new_server
        ),
        (
            EcallUserSubmit,
            (&UserSubmissionReq, &SealedSigPrivKey),
            (RoundSubmissionBlob, SealedSharedSecretDb),
            user_submit
        ),
        (
            EcallAddToAggregate,
            (
                &RoundSubmissionBlob,
                &SignedPartialAggregate,
                &Option<BTreeSet<RateLimitNonce>>,
                &SealedSigPrivKey
            ),
            (SignedPartialAggregate, Option<BTreeSet<RateLimitNonce>>),
            add_to_agg
        ),
        (
            EcallRecvUserRegistration,
            // input:
            (&SignedPubKeyDb, &SealedSharedSecretDb, &SealedKemPrivKey, &UserRegistrationBlob),
            // output: updated SignedPubKeyDb, SealedSharedSecretDb
            (SignedPubKeyDb, SealedSharedSecretDb),
            recv_user_reg
        ),
        (
            EcallRecvUserRegistrationBatch,
            (&SignedPubKeyDb, &SealedKemPrivKey, &[UserRegistrationBlob]),
            (SignedPubKeyDb, SealedSharedSecretDb),
            recv_user_reg_batch
        ),
        (
            EcallUnblindAggregate,
            (&RoundSubmissionBlob,&SealedSigPrivKey,&SealedSharedSecretDb),
            (UnblindedAggregateShareBlob, SealedSharedSecretDb),
            unblind_aggregate
        ),
        (
            EcallUnblindAggregatePartial,
            (u32,&SealedSharedSecretDb,&BTreeSet<EntityId>),
            RoundSecret,
            unblind_aggregate_partial
        ),
        (
            EcallUnblindAggregateMerge,
            (&RoundSubmissionBlob,&[RoundSecret], &SealedSigPrivKey, &SealedSharedSecretDb),
            (UnblindedAggregateShareBlob, SealedSharedSecretDb),
            unblind_aggregate_merge
        ),
        (
            EcallDeriveRoundOutput,
            (&SealedSigPrivKey,&[UnblindedAggregateShareBlob]),
            RoundOutput,
            derive_round_output
        ),
        (
            EcallRecvAggregatorRegistration,
            (&SignedPubKeyDb, &AggRegistrationBlob),
            SignedPubKeyDb,
            recv_aggregator_registration
        ),
        (
            EcallRecvServerRegistration,
            (&SignedPubKeyDb, &ServerRegistrationBlob),
            SignedPubKeyDb,
            recv_server_registration
        ),
    }
}

use itertools::Itertools;
use std::iter::FromIterator;
use std::sync::mpsc;
use std::thread;

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

        let start_time = std::time::Instant::now();
        let enclave = SgxEnclave::create(
            enclave_path,
            debug,
            &mut launch_token,
            &mut launch_token_updated,
            &mut misc_attr,
        )
        .map_err(EnclaveError::SgxError)?;

        info!(
            "============== enclave created. took {}us",
            start_time.elapsed().as_micros()
        );
        Ok(Self { enclave })
    }

    pub fn destroy(self) {
        self.enclave.destroy();
    }

    /// This method can be used for creating signing keys and KEM private keys.
    /// Use unseal_to_pubkey to unseal the key and compute its public key.
    /// Returns (sealed sk, AttestedPublicKey)
    fn new_sgx_protected_key(&self, role: String) -> EnclaveResult<(Vec<u8>, AttestedPublicKey)> {
        Ok(ecall_allowed::new_sgx_keypair(self.enclave.geteid(), role)?)
    }

    /// Returns the public key corresponding to the sealed secret key
    pub fn unseal_to_public_key_on_p256(
        &self,
        sealed_private_key: &Vec<u8>,
    ) -> EnclaveResult<SgxProtectedKeyPub> {
        Ok(ecall_allowed::unseal_to_public_key(
            self.enclave.geteid(),
            sealed_private_key,
        )?)
    }

    /// Given a message, constructs a round message for sending to an aggregator
    /// SGX will
    ///     1. Check the signature on the preivous round output against a signing key (might have to change API a bit for that)
    ///     2. Check that the current round is prev_round+1
    ///     3. Use the derived slot for the given message
    ///     4. Make a new footprint reservation for this round
    ///
    /// This function returns the message as well as the ratcheted shared secrets.
    ///
    /// Error handling:
    ///
    /// If scheduling failed (e.g., due to collision) this will return
    /// Err(EnclaveLogicError(SGX_ERROR_SERVICE_UNAVAILABLE)). Higher level application should
    /// retry, for example, in the next round.
    pub fn user_submit_round_msg(
        &self,
        submission_req: &UserSubmissionReq,
        sealed_usk: &SealedSigPrivKey,
    ) -> EnclaveResult<(RoundSubmissionBlob, SealedSharedSecretDb)> {
        Ok(ecall_allowed::user_submit(
            self.enclave.geteid(),
            (submission_req, sealed_usk),
        )?)
    }

    /// Makes an empty aggregation state for the given round and wrt the given anytrust nodes
    pub fn new_aggregate(
        &self,
        _round: u32,
        _anytrust_group_id: &EntityId,
    ) -> EnclaveResult<SignedPartialAggregate> {
        // A new aggregator is simply an empty blob
        Ok(Default::default())
    }

    /// Adds the given input from a user to the given partial aggregate
    /// Note: if marshalled_current_aggregation is empty (len = 0), an empty aggregation is created
    ///  and the signed message is aggregated into that.
    pub fn add_to_aggregate(
        &self,
        agg: &mut SignedPartialAggregate,
        observed_nonces: &mut Option<BTreeSet<RateLimitNonce>>,
        new_input: &RoundSubmissionBlob,
        signing_key: &SealedSigPrivKey,
    ) -> EnclaveResult<()> {
        let (new_agg, new_observed_nonces) = ecall_allowed::add_to_agg(
            self.enclave.geteid(),
            (new_input, agg, observed_nonces, signing_key),
        )?;

        // Update the agg and nonces
        *agg = new_agg;
        *observed_nonces = new_observed_nonces;

        Ok(())
    }

    /// Constructs an aggregate message from the given state. The returned blob is to be sent to
    /// the parent aggregator or an anytrust server.
    /// Note: this is an identity function because SignedPartialAggregate and RoundSubmissionBlob
    /// are exact the same thing.
    pub fn finalize_aggregate(
        &self,
        agg: &SignedPartialAggregate,
    ) -> EnclaveResult<RoundSubmissionBlob> {
        return Ok(agg.clone());
    }

    /// XORs the shared secrets into the given aggregate. Returns the server's share of the
    /// unblinded aggregate as well as the ratcheted shared secrets.
    ///
    /// This is invoked by the root anytrust server.
    pub fn unblind_aggregate_single_thread(
        &self,
        toplevel_agg: &RoundSubmissionBlob,
        signing_key: &SealedSigPrivKey,
        shared_secrets: &SealedSharedSecretDb,
    ) -> EnclaveResult<(UnblindedAggregateShareBlob, SealedSharedSecretDb)> {
        ecall_allowed::unblind_aggregate(
            self.enclave.geteid(),
            (toplevel_agg, signing_key, shared_secrets),
        )
    }

    /// The multi thread version
    pub fn unblind_aggregate_mt(
        &self,
        toplevel_agg: &AggregatedMessage,
        signing_key: &SealedSigPrivKey,
        shared_secrets: &SealedSharedSecretDb,
        n_threads: usize,
    ) -> EnclaveResult<(UnblindedAggregateShareBlob, SealedSharedSecretDb)> {
        let start = Instant::now();
        let chunk_size = (toplevel_agg.user_ids.len() + n_threads - 1) / n_threads;
        assert_ne!(chunk_size, 0);

        let eid = self.enclave.geteid();
        let round = shared_secrets.round;

        // make a mpsc channel
        let (tx, rx) = mpsc::channel();

        // // partition the user ids into N batches
        let user_keys: Vec<EntityId> = toplevel_agg.user_ids.iter().cloned().collect();
        for uks in &user_keys.into_iter().chunks(chunk_size) {
            let uks_vec = uks.collect_vec();

            let db_cloned = shared_secrets.clone();
            let tx_cloned = mpsc::Sender::clone(&tx);

            thread::spawn(move || {
                info!("thread working on {} ids", uks_vec.len());
                let user_ids: BTreeSet<EntityId> = BTreeSet::from_iter(uks_vec.into_iter());
                let rs =
                    ecall_allowed::unblind_aggregate_partial(eid, (round, &db_cloned, &user_ids))
                        .unwrap();
                tx_cloned.send(rs).unwrap();
            });
        }

        info!("========= set up threads after {:?}", start.elapsed());

        drop(tx);

        let round_secrets: Vec<RoundSecret> = rx.iter().collect();
        info!("========= threads join after {:?}", start.elapsed());

        let result = ecall_allowed::unblind_aggregate_merge(
            self.enclave.geteid(),
            (toplevel_agg, &round_secrets, signing_key, shared_secrets),
        );

        info!(
            "========= {} round secrets merged after {:?}.",
            round_secrets.len(),
            start.elapsed()
        );

        result
    }

    /// The multi thread version, with parameters taken from config file
    pub fn unblind_aggregate(
        &self,
        toplevel_agg: &AggregatedMessage,
        signing_key: &SealedSigPrivKey,
        shared_secrets: &SealedSharedSecretDb,
    ) -> EnclaveResult<(UnblindedAggregateShareBlob, SealedSharedSecretDb)> {
        self.unblind_aggregate_mt(
            toplevel_agg,
            signing_key,
            shared_secrets,
            interface::N_THREADS_DERIVE_ROUND_SECRET,
        )
    }

    /// Derives the final round output given all the shares of the unblinded aggregates
    pub fn derive_round_output(
        &self,
        sealed_sig_sk: &SealedSigPrivKey,
        server_aggs: &[UnblindedAggregateShareBlob],
    ) -> EnclaveResult<RoundOutput> {
        ecall_allowed::derive_round_output(self.enclave.geteid(), (sealed_sig_sk, server_aggs))
    }

    /// Create a new TEE protected secret key. Derives shared secrets with all the given KEM pubkeys.
    /// This function
    /// 1. Verify the enclave attestations on the packages
    /// 2. Use the KEM pubkeys to derive the shared secrets.
    /// TODO: what should it do with the signing keys?
    pub fn new_user(
        &self,
        server_pks: &[ServerPubKeyPackage],
    ) -> EnclaveResult<(
        SealedSharedSecretDb,
        SealedSigPrivKey,
        EntityId,
        UserRegistrationBlob,
    )> {
        let u = ecall_allowed::new_user(self.enclave.geteid(), server_pks)?;
        Ok((u.0, u.1, EntityId::from(&u.2), u.2))
    }

    /// Create a new TEE protected secret key for an aggregator.
    /// Returns sealed private key, entity id, and an AggRegistrationBlob that contains the
    /// attestation information to send to anytrust nodes.
    pub fn new_aggregator(
        &self,
    ) -> EnclaveResult<(SealedSigPrivKey, EntityId, AggRegistrationBlob)> {
        let (sealed_sk, attested_pk) = self.new_sgx_protected_key("agg".to_string())?;
        Ok((
            SealedSigPrivKey(sealed_sk),
            EntityId::from(&attested_pk.pk),
            AggRegistrationBlob(attested_pk),
        ))
    }

    /// Create new TEE protected secret keys. Returns ServerRegistrationBlob that contains the
    /// pubkey and attestation information to send to other anytrust nodes.
    pub fn new_server(
        &self,
    ) -> EnclaveResult<(
        SealedSigPrivKey,
        SealedKemPrivKey,
        EntityId,
        ServerRegistrationBlob,
    )> {
        let s = ecall_allowed::new_server(self.enclave.geteid(), ())?;

        Ok((s.0, s.1, EntityId::from(&s.2), s.2))
    }

    /// Verifies and adds the given user registration blob to the database of pubkeys and
    /// shared secrets
    /// Called by a server
    pub fn recv_user_registration(
        &self,
        pubkeys: &mut SignedPubKeyDb,
        shared_secrets: &mut SealedSharedSecretDb,
        decap_key: &SealedKemPrivKey,
        input_blob: &UserRegistrationBlob,
    ) -> EnclaveResult<()> {
        let (new_pubkey_db, new_secrets_db) = ecall_allowed::recv_user_reg(
            self.enclave.geteid(),
            (pubkeys, shared_secrets, decap_key, input_blob),
        )?;

        pubkeys.users.clear();
        pubkeys.users.extend(new_pubkey_db.users);

        shared_secrets.db.clear();
        shared_secrets.db.extend(new_secrets_db.db);

        Ok(())
    }

    pub fn recv_user_registration_batch(
        &self,
        pubkeys: &mut SignedPubKeyDb,
        shared_secrets: &mut SealedSharedSecretDb,
        decap_key: &SealedKemPrivKey,
        input_blob: &[UserRegistrationBlob],
    ) -> EnclaveResult<()> {
        let (new_pubkey_db, new_secrets_db) = ecall_allowed::recv_user_reg_batch(
            self.enclave.geteid(),
            (pubkeys, decap_key, input_blob),
        )?;

        pubkeys.users = new_pubkey_db.users;
        shared_secrets.db = new_secrets_db.db;

        Ok(())
    }

    /// Verifies and adds the given aggregator registration blob to the database of pubkeys
    pub fn recv_aggregator_registration(
        &self,
        pubkeys: &mut SignedPubKeyDb,
        input_blob: &AggRegistrationBlob,
    ) -> EnclaveResult<()> {
        let new_db = ecall_allowed::recv_aggregator_registration(
            self.enclave.geteid(),
            (pubkeys, input_blob),
        )?;

        pubkeys.aggregators.clear();
        pubkeys.aggregators.extend(new_db.aggregators);

        Ok(())
    }

    /// Verifies and adds the given server registration blob to the database of pubkeys
    pub fn recv_server_registration(
        &self,
        pubkeys: &mut SignedPubKeyDb,
        input_blob: &ServerRegistrationBlob,
    ) -> EnclaveResult<()> {
        let new_db =
            ecall_allowed::recv_server_registration(self.enclave.geteid(), (pubkeys, input_blob))?;

        pubkeys.servers.clear();
        pubkeys.servers.extend(new_db.servers);

        Ok(())
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

    use env_logger::{Builder, Env};
    use hex::FromHex;
    use interface::{
        DcMessage, EntityId, SealedFootprintTicket, SgxProtectedKeyPub, UserSubmissionReq,
        DC_NET_MESSAGE_LENGTH, SEALED_SGX_SIGNING_KEY_LENGTH, USER_ID_LENGTH,
    };
    use log::*;
    use sgx_types::SGX_ECP256_KEY_SIZE;
    use std::time::{Duration, Instant};
    use std::{collections::BTreeSet, vec};
    use tokio::select_variant;

    fn init_logger() {
        let env = Env::default()
            .filter_or("RUST_LOG", "debug")
            .write_style_or("RUST_LOG_STYLE", "always");

        let _ = Builder::from_env(env).try_init();
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn key_seal_unseal() {
        init_logger();

        log::info!("log in test");
        let enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();
    }

    /// create n server public keys
    fn create_server_pubkeys(enc: &DcNetEnclave, n: i32) -> Vec<ServerPubKeyPackage> {
        let mut pks = Vec::new();

        for i in 0..n {
            let s = enc.new_server().unwrap();
            pks.push(s.3);
        }

        pks
    }

    #[test]
    fn user_submit_round_msg() {
        init_logger();
        let enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();

        // create server public keys
        let spks = create_server_pubkeys(&enc, 10);
        let (user_reg_shared_secrets, user_reg_sealed_key, user_reg_uid, user_reg_proof) =
            enc.new_user(&spks).unwrap();

        let msg = UserMsg::TalkAndReserve {
            msg: DcMessage(vec![1u8; DC_NET_MESSAGE_LENGTH]),
            prev_round_output: RoundOutput::default(),
            times_participated: 0,
        };

        let req_1 = UserSubmissionReq {
            user_id: user_reg_uid,
            anytrust_group_id: user_reg_shared_secrets.anytrust_group_id(),
            round: 0,
            msg,
            shared_secrets: user_reg_shared_secrets,
            server_pks: spks,
        };

        let (resp_1, _) = enc
            .user_submit_round_msg(&req_1, &user_reg_sealed_key)
            .unwrap();

        // if we set round to 1, this should fail because the previous round output is empty
        let mut req_round_1 = req_1.clone();
        req_round_1.round = 1;

        assert!(enc
            .user_submit_round_msg(&req_round_1, &user_reg_sealed_key)
            .is_err());

        enc.destroy();
    }

    #[test]
    fn user_reserve_slot() {
        init_logger();
        let enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();

        // create server public keys
        let spks = create_server_pubkeys(&enc, 10);
        let (user_reg_shared_secrets, user_reg_sealed_key, user_reg_uid, user_reg_proof) =
            enc.new_user(&spks).unwrap();

        let msg = UserMsg::Reserve {
            times_participated: 0,
        };

        let req_1 = UserSubmissionReq {
            user_id: user_reg_uid,
            anytrust_group_id: user_reg_shared_secrets.anytrust_group_id(),
            round: 0,
            msg,
            shared_secrets: user_reg_shared_secrets,
            server_pks: spks,
        };

        let (resp_1, _) = enc
            .user_submit_round_msg(&req_1, &user_reg_sealed_key)
            .unwrap();

        enc.destroy();
    }

    #[test]
    fn aggregation() {
        init_logger();
        let enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();

        // create server public keys
        let num_of_servers = 10;
        let server_pks = create_server_pubkeys(&enc, num_of_servers);
        log::info!("created {} server keys", num_of_servers);

        // create a fake user
        let (user_reg_shared_secrets, user_reg_sealed_key, user_reg_uid, user_reg_proof) =
            enc.new_user(&server_pks).unwrap();

        log::info!("user {:?} created", user_reg_uid);

        let msg1 = UserMsg::TalkAndReserve {
            msg: DcMessage(vec![1u8; DC_NET_MESSAGE_LENGTH]),
            prev_round_output: RoundOutput::default(),
            times_participated: 0,
        };

        let req_1 = UserSubmissionReq {
            user_id: user_reg_uid,
            anytrust_group_id: user_reg_shared_secrets.anytrust_group_id(),
            round: 0,
            msg: msg1,
            shared_secrets: user_reg_shared_secrets,
            server_pks: server_pks.clone(),
        };

        log::info!("submitting for user {:?}", req_1.user_id);

        let (resp_1, _) = enc
            .user_submit_round_msg(&req_1, &user_reg_sealed_key)
            .unwrap();

        // SealedSigPrivKey, EntityId, AggRegistrationBlob
        let agg = enc.new_aggregator().expect("agg");

        log::info!("aggregator {:?} created", agg.1);

        let mut empty_agg = enc.new_aggregate(0, &EntityId::default()).unwrap();
        let mut observed_nonces = Some(BTreeSet::new());
        enc.add_to_aggregate(&mut empty_agg, &mut observed_nonces, &resp_1, &agg.0)
            .unwrap();

        // this should error because user is already in
        assert!(enc
            .add_to_aggregate(&mut empty_agg, &mut observed_nonces, &resp_1, &agg.0)
            .is_err());

        log::info!("error expected");

        let user_2 = enc.new_user(&server_pks).unwrap();

        let msg2 = UserMsg::TalkAndReserve {
            msg: DcMessage(vec![2u8; DC_NET_MESSAGE_LENGTH]),
            prev_round_output: RoundOutput::default(),
            times_participated: 0,
        };

        let req_2 = UserSubmissionReq {
            user_id: user_2.2,
            anytrust_group_id: user_2.0.anytrust_group_id(),
            round: 0,
            msg: msg2,
            shared_secrets: user_2.0,
            server_pks,
        };
        let (resp_2, _) = enc.user_submit_round_msg(&req_2, &user_2.1).unwrap();

        enc.add_to_aggregate(&mut empty_agg, &mut observed_nonces, &resp_2, &agg.0)
            .unwrap();

        // Ensure we saw two distinct nonces
        assert_eq!(observed_nonces.unwrap().len(), 2);

        enc.destroy();
    }

    #[test]
    fn new_user() {
        init_logger();

        let enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();
        let pks = create_server_pubkeys(&enc, 2);
        let (user_reg_shared_secrets, user_reg_sealed_key, user_reg_uid, user_reg_proof) =
            enc.new_user(&pks).unwrap();

        let pk = enc
            .unseal_to_public_key_on_p256(&user_reg_sealed_key.0)
            .unwrap();
        assert_eq!(EntityId::from(&pk), user_reg_uid);

        enc.destroy();
    }

    #[test]
    fn new_aggregator() {
        let enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();

        let (agg_sealed_key, agg_id, agg_reg_proof) = enc.new_aggregator().unwrap();

        let pk = enc.unseal_to_public_key_on_p256(&agg_sealed_key.0).unwrap();
        assert_eq!(EntityId::from(&pk), agg_id);

        enc.destroy();
    }

    use interface::*;

    fn create_n_servers(
        n: usize,
        enclave: &DcNetEnclave,
    ) -> Vec<(
        SealedSigPrivKey,
        SealedKemPrivKey,
        EntityId,
        ServerRegistrationBlob,
    )> {
        let mut servers = Vec::new();
        for _ in 0..n {
            servers.push(enclave.new_server().unwrap());
        }

        servers
    }

    #[test]
    fn server_recv_user_reg() {
        init_logger();

        let enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();
        let servers = create_n_servers(2, &enc);

        let mut server_pks = Vec::new();
        for (_, _, _, k) in servers.iter().cloned() {
            server_pks.push(k)
        }

        let user = enc.new_user(&server_pks).expect("user");

        info!("user created {:?}", user.2);

        let server_1 = &servers[0];

        let mut pk_db = Default::default();
        let mut secret_db = Default::default();

        enc.recv_user_registration(&mut pk_db, &mut secret_db, &server_1.1, &user.3)
            .unwrap();
    }

    // fn test_run(n_server: usize, n_users: usize) {
    //     init_logger();
    //
    //     let enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();
    //     let servers = create_n_servers(n_server, &enc);
    // }
    //
    fn whole_thing_n_thread(n: usize) {
        init_logger();

        let enc = DcNetEnclave::init(TEST_ENCLAVE_PATH).unwrap();

        // create server public keys
        let num_of_servers = 1;
        let num_of_users = interface::DC_NET_N_SLOTS;

        let servers = create_n_servers(num_of_servers, &enc);
        let mut server_pks = Vec::new();
        for (_, _, _, k) in servers.iter().cloned() {
            server_pks.push(k)
        }

        info!("created {} server keys", num_of_servers);

        // create a bunch of fake user
        let users = (0..num_of_users)
            .map(|_| enc.new_user(&server_pks).unwrap())
            .collect::<Vec<_>>();
        let user_pks = (0..num_of_users)
            .map(|i| users[i].3.clone())
            .collect::<Vec<_>>();

        info!("{} user created", num_of_users);

        // create aggregator
        let aggregator = enc.new_aggregator().expect("agg");
        let mut empty_agg = enc.new_aggregate(0, &EntityId::default()).unwrap();
        let mut observed_nonces = Some(BTreeSet::new());

        info!("🏁 aggregator {:?} created", aggregator.1);

        // server state
        let mut server_pk_db = vec![SignedPubKeyDb::default(); num_of_servers];
        let mut server_shared_db = vec![SealedSharedSecretDb::default(); num_of_servers];

        // register users
        info!("============== registering users");
        let start = Instant::now();

        for (i, s) in servers.iter().enumerate() {
            let mut pk_db = SignedPubKeyDb::default();
            let mut shared_db = SealedSharedSecretDb::default();

            // register the aggregator
            enc.recv_aggregator_registration(&mut pk_db, &aggregator.2)
                .unwrap();

            enc.recv_user_registration_batch(&mut pk_db, &mut shared_db, &s.1, &user_pks)
                .unwrap();

            info!("========= registration done at server {}", i);

            server_pk_db[i] = pk_db;
            server_shared_db[i] = shared_db;
        }

        info!(
            "============== all user have registered. used {:?}",
            start.elapsed()
        );
        let start = Instant::now();

        for i in 0..2 {
            let user = &users[i];

            let dc_msg = DcMessage(vec![(i + 1) as u8; DC_NET_MESSAGE_LENGTH]);
            let msg0 = UserMsg::TalkAndReserve {
                msg: dc_msg.clone(),
                prev_round_output: RoundOutput::default(),
                times_participated: 0,
            };

            let req_0 = UserSubmissionReq {
                user_id: user.2,
                anytrust_group_id: user.0.anytrust_group_id(),
                round: 0,
                msg: msg0,
                shared_secrets: user.0.clone(),
                server_pks: server_pks.clone(),
            };

            let (resp_0, _) = enc.user_submit_round_msg(&req_0, &user.1).unwrap();

            log::info!("🏁 user {} submitted", i);

            enc.add_to_aggregate(&mut empty_agg, &mut observed_nonces, &resp_0, &aggregator.0)
                .unwrap();
        }

        info!("========= all user submitted. Took {:?}", start.elapsed());

        // finalize the aggregate
        // let final_agg_0 = enc.finalize_aggregate(&empty_agg).unwrap();

        // fake a lot of user ids
        let final_agg_0 = RoundSubmissionBlob {
            round: 0,
            anytrust_group_id: empty_agg.anytrust_group_id,
            user_ids: server_shared_db[0].db.keys().map(EntityId::from).collect(),
            rate_limit_nonce: None,
            aggregated_msg: empty_agg.aggregated_msg,
            tee_sig: Default::default(),
            tee_pk: Default::default(),
        };

        info!("========= decryption begins");
        let start = Instant::now();

        // decryption
        let mut decryption_shares = Vec::new();
        for (i, s) in servers.iter().enumerate() {
            // unblind
            let (unblined_agg, _) = enc
                .unblind_aggregate_mt(&final_agg_0, &s.0, &server_shared_db[i], n)
                .unwrap();
            decryption_shares.push(unblined_agg);
        }

        info!(
            "🏁 {} decryption shares obtained. Each {} bytes. Took {:?}",
            decryption_shares.len(),
            decryption_shares[0].0.len(),
            start.elapsed(),
        );

        // aggregate final shares
        // suppose the first server is the leader
        let round_output_r0 = enc
            .derive_round_output(&servers[0].0, &decryption_shares)
            .unwrap();
        // info!("✅ round_output {:?}", round_output_r0);
        info!("✅ round_output");

        // let msg1 = UserMsg::TalkAndReserve {
        //     msg: dc_msg,
        //     prev_round_output: round_output_r0,
        //     times_participated: 1,
        // };
        // let mut req_r1 = req_0.clone();
        // req_r1.msg = msg1;

        // info!("🏁 starting round 1");
        // let (resp_1, _) = enc.user_submit_round_msg(&req_r1, &user.1).unwrap();
    }

    #[test]
    fn whole_thing() {
        whole_thing_n_thread(10)
    }

    #[test]
    fn reference() {
        whole_thing_n_thread(1)
    }
}
