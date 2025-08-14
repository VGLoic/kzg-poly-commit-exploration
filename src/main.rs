use clap::{Parser, Subcommand};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{BufReader, Write},
    str::FromStr,
};
use thiserror::Error;

use kzg_poly_commit_exploration::{
    curves::G1Point,
    polynomial::{Evaluation, Polynomial},
    trusted_setup,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Perform a trusted setup ceremony and write the artifacts in './artifacts/setup.json'.
    ///
    /// Artifacts are genetated until degree 9.
    TrustedSetup {},
    /// Commit to a polynomial using the trusted setup artifacts
    Commit {
        /// Coefficients of the polynomial in ascending degree, starting from the degree zero.
        ///
        /// Degree up to 9 is supported.
        #[arg(long_help, num_args = 1..)]
        coefficients: Vec<i128>,
    },
    /// Evaluate the committed polynomial at an input point and generate the associated Kate proof.
    Evaluate {
        /// Input point
        #[arg()]
        x: i128,
    },
    /// Verify the previous evaluation with its proof
    VerifyEvaluation {},
}

fn main() {
    let cli = Cli::parse();

    let default_log_level = match &cli.debug {
        0 => log::Level::Info,
        1 => log::Level::Debug,
        _ => log::Level::Trace,
    };

    if let Err(err) = dotenvy::dotenv() {
        if !err.not_found() {
            panic!("Error while loading .env file: {err}")
        }
    }

    let log_level = match std::env::var("LOG_LEVEL").ok() {
        Some(v) => log::Level::from_str(v.as_str()).unwrap_or(default_log_level),
        None => default_log_level,
    };

    if let Err(err) = simple_logger::init_with_level(log_level) {
        panic!("Failed to initialize logging, got error: {err}");
    }

    match &cli.command {
        Some(cmd) => {
            if let Err(e) = cmd.run() {
                panic!("Command execution failed with error: {e}");
            }
        }
        None => {
            log::warn!("No command has been input")
        }
    }
}

#[derive(Error, Debug)]
enum CliError {
    #[error("Unhandled error: {0}")]
    UnhandledError(#[from] anyhow::Error),
}

impl From<std::io::Error> for CliError {
    fn from(value: std::io::Error) -> Self {
        CliError::UnhandledError(anyhow::Error::new(value))
    }
}

const ARTIFACTS_FOLDER_PATH: &str = "./artifacts";
const SETUP_ARTIFACTS_PATH: &str = "./artifacts/setup.json";
const COMMITMENT_ARTIFACTS_PATH: &str = "./artifacts/commitment.json";
const EVALUATION_ARTIFACTS_PATH: &str = "./artifacts/evaluation.json";

const MAX_DEGREE: u32 = 9;

impl Commands {
    fn run(&self) -> Result<(), CliError> {
        match &self {
            Commands::TrustedSetup {} => {
                log::info!("Starting the trusted setup ceremony");

                if !fs::exists(ARTIFACTS_FOLDER_PATH)? {
                    fs::create_dir(ARTIFACTS_FOLDER_PATH)?;
                }
                if fs::exists(SETUP_ARTIFACTS_PATH)? {
                    fs::remove_file(SETUP_ARTIFACTS_PATH)?;
                }
                let mut file = fs::File::create(SETUP_ARTIFACTS_PATH)?;

                let mut s_be_bytes = [0; 32];
                rand::rng().fill_bytes(&mut s_be_bytes);

                let setup_artifacts: Vec<_> =
                    trusted_setup::SetupArtifactsGenerator::new(s_be_bytes)
                        .take((MAX_DEGREE + 1) as usize)
                        .collect();

                let stringified_artifacts =
                    serde_json::to_string(&setup_artifacts).map_err(anyhow::Error::from)?;

                file.write_all(stringified_artifacts.as_bytes())?;

                log::info!(
                    "Trusted setup ceremony successfully performed. Artifacts have been written in \"{SETUP_ARTIFACTS_PATH}\""
                );

                Ok(())
            }
            Commands::Commit { coefficients } => {
                let polynomial = Polynomial::try_from(coefficients.as_slice())?;

                let polynomial_displayed = polynomial.to_string();

                if polynomial.degree() > MAX_DEGREE {
                    return Err(anyhow::anyhow!(
                        "Only polynomials up to degree {MAX_DEGREE} are supported"
                    )
                    .into());
                }

                log::info!(
                    "Starting to commit to the polynomial P(x) = \"{polynomial_displayed}\""
                );

                if !fs::exists(SETUP_ARTIFACTS_PATH)? {
                    return Err(anyhow::anyhow!(
                        "Trusted setup artifacts have not been found, generate them beforehand."
                    )
                    .into());
                }

                let file = fs::File::open(SETUP_ARTIFACTS_PATH)?;
                let reader = BufReader::new(file);

                let setup_artifacts: Vec<trusted_setup::SetupArtifact> =
                    serde_json::from_reader(reader).map_err(anyhow::Error::from)?;

                let commitment = polynomial.commit(&setup_artifacts)?;

                let commitment_artifact = serde_json::to_string(&CommitmentArtifact {
                    polynomial,
                    commitment,
                })
                .map_err(anyhow::Error::from)?;

                if fs::exists(COMMITMENT_ARTIFACTS_PATH)? {
                    fs::remove_file(COMMITMENT_ARTIFACTS_PATH)?;
                }
                let mut file = fs::File::create(COMMITMENT_ARTIFACTS_PATH)?;
                file.write_all(commitment_artifact.as_bytes())?;

                log::info!(
                    "Commitment to the polynomial \"P(x) = {polynomial_displayed}\" has been successfully generated."
                );

                Ok(())
            }
            Commands::Evaluate { x } => {
                log::info!(
                    "Starting to evaluate the committed polynomial at input point \"x = {x}\""
                );

                if !fs::exists(SETUP_ARTIFACTS_PATH)? {
                    return Err(anyhow::anyhow!(
                        "Trusted setup artifacts have not been found, generate them beforehand."
                    )
                    .into());
                }

                let file = fs::File::open(SETUP_ARTIFACTS_PATH)?;
                let reader = BufReader::new(file);

                let setup_artifacts: Vec<trusted_setup::SetupArtifact> =
                    serde_json::from_reader(reader).map_err(anyhow::Error::from)?;

                if !fs::exists(COMMITMENT_ARTIFACTS_PATH)? {
                    return Err(anyhow::anyhow!(
                        "Commitment artifact has not been found, generate it beforehand."
                    )
                    .into());
                }
                let file = fs::File::open(COMMITMENT_ARTIFACTS_PATH)?;
                let reader = BufReader::new(file);
                let commitment_artifact: CommitmentArtifact =
                    serde_json::from_reader(reader).map_err(anyhow::Error::from)?;

                let evaluation = commitment_artifact.polynomial.evaluate(x)?;
                let proof =
                    evaluation.generate_proof(&commitment_artifact.polynomial, &setup_artifacts)?;

                // REMIND ME
                let evaluation_artifact = serde_json::to_string(&EvaluationArtifact {
                    evaluation: evaluation.clone(),
                    proof,
                })
                .map_err(anyhow::Error::from)?;

                if fs::exists(EVALUATION_ARTIFACTS_PATH)? {
                    fs::remove_file(EVALUATION_ARTIFACTS_PATH)?;
                }
                let mut file = fs::File::create(EVALUATION_ARTIFACTS_PATH)?;
                file.write_all(evaluation_artifact.as_bytes())?;

                log::info!(
                    "Evaluation successful for polynomial: \"P(x) = {}\" at point \"x = {x}\" with \"P({x}) = {}\"",
                    commitment_artifact.polynomial,
                    evaluation.result
                );

                Ok(())
            }
            Commands::VerifyEvaluation {} => {
                log::info!("Starting to verify the previous polynomial evaluation");

                if !fs::exists(SETUP_ARTIFACTS_PATH)? {
                    return Err(anyhow::anyhow!(
                        "Trusted setup artifacts have not been found, generate them beforehand."
                    )
                    .into());
                }

                let file = fs::File::open(SETUP_ARTIFACTS_PATH)?;
                let reader = BufReader::new(file);

                let setup_artifacts: Vec<trusted_setup::SetupArtifact> =
                    serde_json::from_reader(reader).map_err(anyhow::Error::from)?;

                if !fs::exists(COMMITMENT_ARTIFACTS_PATH)? {
                    return Err(anyhow::anyhow!(
                        "Commitment artifact has not been found, generate it beforehand."
                    )
                    .into());
                }
                let file = fs::File::open(COMMITMENT_ARTIFACTS_PATH)?;
                let reader = BufReader::new(file);
                let commitment_artifact: CommitmentArtifact =
                    serde_json::from_reader(reader).map_err(anyhow::Error::from)?;

                if !fs::exists(EVALUATION_ARTIFACTS_PATH)? {
                    return Err(anyhow::anyhow!(
                        "Evaluation artifact has not been found, generate it beforehand."
                    )
                    .into());
                }
                let file = fs::File::open(EVALUATION_ARTIFACTS_PATH)?;
                let reader = BufReader::new(file);
                let evaluation_artifact: EvaluationArtifact =
                    serde_json::from_reader(reader).map_err(anyhow::Error::from)?;

                let is_proof_ok = evaluation_artifact.evaluation.verify_proof(
                    &evaluation_artifact.proof,
                    &commitment_artifact.commitment,
                    &setup_artifacts,
                )?;

                if !is_proof_ok {
                    return Err(anyhow::anyhow!(
                        "The proof associated to the evaluation is incorrect."
                    )
                    .into());
                }

                log::info!(
                    "Successfully verified evaluation for polynomial \"P(x) = {}\" at point \"x = {}\" with \"P({}) = {}\"",
                    commitment_artifact.polynomial,
                    evaluation_artifact.evaluation.point,
                    evaluation_artifact.evaluation.point,
                    evaluation_artifact.evaluation.result
                );

                Ok(())
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CommitmentArtifact {
    polynomial: Polynomial,
    commitment: G1Point,
}

#[derive(Debug, Serialize, Deserialize)]
struct EvaluationArtifact {
    evaluation: Evaluation,
    proof: G1Point,
}
