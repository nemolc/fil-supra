use filecoin_proofs_api::{seal, SectorId};
use safer_ffi::prelude::*;
use std::{path::PathBuf, str::Utf8Error, path::Path};
use std::fmt::format;
use std::io::Read;
use std::sync::Once;
use serde::{Deserialize, Serialize};
use data_encoding::{BASE64, HEXLOWER};
use clap::{arg, Arg, ArgAction, ArgMatches, Command};
use clap::builder::Str;
use integer_encoding::VarInt;
use std::string::String;
use anyhow::anyhow;

static LOG_INIT: Once = Once::new();

/// Ensures the logger is initialized.
pub fn init_log() {
    LOG_INIT.call_once(|| {
        fil_logger::init();
    });
}


fn main() {
    init_log();
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("c2", sub_matches)) => {
            let p2out = c2_action(sub_matches);
        }

        _ => unreachable!()
    }
}

fn cli() -> Command {
    Command::new("fil-supra")
        .about("测试 filecoin supra-seal")
        .subcommand(
            Command::new("c2").about("run c2")
                .arg(Arg::new("c1out.json").required(true))
                .arg(Arg::new("prover_id").required(true).action(ArgAction::Set).value_parser(clap::value_parser!(u64)))
        )
}

fn c2_action(cmd: &ArgMatches) -> anyhow::Result<()> {
    let c1json = std::fs::read(cmd.get_one::<String>("c1out.json").ok_or(anyhow!("required"))?)?;
    let c1out: C1out = serde_json::from_slice(&c1json)?;
    let phase1Out = BASE64.decode(c1out.Phase1Out.as_bytes())?;

    let prover_id = {
        let mut prover = cmd.get_one::<u64>("prover_id").unwrap().encode_var_vec();
        let mut prover_id: [u8; 32] = [0; 32];

        for (i, v) in prover.iter().enumerate() {
            prover_id[i] = *v
        }

        prover_id
    };

    log::debug!("commit2: start" );
    let result = c2(phase1Out.as_slice(), c1out.SectorNum, prover_id);
    log::debug!("commit2: end" );

    result.map(|v| println!("proof: {}\n", HEXLOWER.encode(v.as_slice())))
        .map_err(|err| println!("err: {:?}\n", err));

    Ok(())
}


fn c2<'a>(
    seal_commit_phase1_output: &'a [u8],
    sector_id: u64,
    prover_id: [u8; 32],
) -> anyhow::Result<Vec<u8>> {
    let scp1o = serde_json::from_slice(&seal_commit_phase1_output)?;
    let result = seal::seal_commit_phase2(scp1o, prover_id, SectorId::from(sector_id))?;

    Ok(result.proof.into_boxed_slice().into())
}

pub fn as_path_buf(bytes: &[u8]) -> std::result::Result<PathBuf, Utf8Error> {
    std::str::from_utf8(bytes).map(Into::into)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct C1out {
    pub SectorNum: u64,
    pub Phase1Out: String,
    pub SectorSize: u64,
}