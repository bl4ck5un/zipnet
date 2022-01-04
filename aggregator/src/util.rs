use crate::agg_state::AggregatorState;

use std::{fs::File, io};

use clap::ArgMatches;
use common::{cli_util, enclave_wrapper::EnclaveError};
use interface::RoundInfo;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub(crate) type Result<T> = core::result::Result<T, AggregatorError>;

#[derive(Debug, Error)]
pub enum AggregatorError {
    #[error("aggregator has not been initialized")]
    Uninitialized,
    #[error("error from enclave")]
    Enclave(#[from] EnclaveError),
    #[error("error from IO")]
    Io(#[from] io::Error),
    #[error("error in serialization/deserialization")]
    Ser(#[from] cli_util::SerializationError),
}

pub(crate) fn load_state(save_path: &str) -> Result<AggregatorState> {
    let save_file = File::open(save_path)?;
    Ok(cli_util::load(save_file)?)
}

pub(crate) fn load_round_info(matches: &ArgMatches) -> Result<RoundInfo> {
    let round_str = matches.value_of("round").unwrap();
    let window_str = matches.value_of("window").unwrap();
    Ok(RoundInfo {
        round: cli_util::parse_u32(&round_str)?,
        window: cli_util::parse_u32(&window_str)?,
    })
}

pub(crate) fn save_state(save_path: &str, state: &AggregatorState) -> Result<()> {
    let save_file = File::create(save_path)?;
    Ok(cli_util::save(save_file, state)?)
}

pub(crate) fn load_from_stdin<D: for<'a> Deserialize<'a>>() -> Result<D> {
    let stdin = std::io::stdin();
    Ok(cli_util::load(stdin)?)
}

pub(crate) fn save_to_stdout<S: Serialize>(val: &S) -> Result<()> {
    let stdout = std::io::stdout();
    cli_util::save(stdout, val)?;
    println!("");
    Ok(())
}
