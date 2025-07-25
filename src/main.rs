use clap::{Parser, Subcommand};
use rand::RngCore;
use std::{fs, io::Write, str::FromStr};
use thiserror::Error;

mod trusted_setup;

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
    /// Perform a trusted setup ceremony and write the artifacts in './artifacts/setup.json'. Artifacts are genetated until degree 9.
    TrustedSetup {},
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
                panic!("Command execution failed with error: {e}")
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

impl Commands {
    fn run(&self) -> Result<(), CliError> {
        match &self {
            Commands::TrustedSetup {} => {
                log::info!("Starting the trusted setup ceremony");

                let artifacts_folder_path = "./artifacts";
                let setup_artifacts_path = format!("{artifacts_folder_path}/setup.json");
                if !fs::exists(artifacts_folder_path)? {
                    fs::create_dir(artifacts_folder_path)?;
                }
                if fs::exists(&setup_artifacts_path)? {
                    fs::remove_file(&setup_artifacts_path)?;
                }
                let mut file = fs::File::create(&setup_artifacts_path)?;

                let mut s_be_bytes = [0; 48];
                rand::rng().fill_bytes(&mut s_be_bytes);

                const MAX_DEGREE: u8 = 9;

                let setup_artifacts: Vec<_> =
                    trusted_setup::SetupArtifactsGenerator::new(s_be_bytes)
                        .take(usize::from(MAX_DEGREE))
                        .collect();

                let stringified_artifacts =
                    serde_json::to_string(&setup_artifacts).map_err(anyhow::Error::from)?;

                file.write_all(stringified_artifacts.as_bytes())?;

                log::info!(
                    "Trusted setup ceremony successfully performed. Artifacts have been written in \"{setup_artifacts_path}\""
                );

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::RngCore;

    use super::trusted_setup::SetupArtifactsGenerator;

    #[test]
    fn test_point_addition_and_scalar_multiplication() {
        unsafe {
            let g1 = blst::blst_p1_generator();

            let mut p1_via_addition = blst::blst_p1::default();
            blst::blst_p1_add_or_double(&mut p1_via_addition, g1, g1);

            let mut p1_via_multiplication = blst::blst_p1::default();
            let scalar_as_bytes = 2_u8.to_be_bytes();
            blst::blst_p1_mult(
                &mut p1_via_multiplication,
                g1,
                scalar_as_bytes.as_ptr(),
                scalar_as_bytes.len() * 8,
            );

            assert!(blst::blst_p1_in_g1(g1), "g1 must be in the first group");
            assert_eq!(
                p1_via_multiplication, p1_via_addition,
                "results must be the same via multiplication and via addition"
            );
            assert_ne!(
                p1_via_multiplication, *g1,
                "result must be different than g1"
            );
            assert!(
                blst::blst_p1_in_g1(&p1_via_multiplication),
                "result must be in first group"
            );
        }
    }

    #[test]
    fn test_compression_and_serialization() {
        unsafe {
            let g1 = blst::blst_p1_generator();

            let mut p1 = blst::blst_p1::default();
            blst::blst_p1_add_or_double(&mut p1, g1, g1);

            let mut compressed_p1 = [0; 48];
            blst::blst_p1_compress(compressed_p1.as_mut_ptr(), &p1);
            let mut uncompressed_p1_affine = blst::blst_p1_affine::default();
            match blst::blst_p1_uncompress(&mut uncompressed_p1_affine, compressed_p1.as_ptr()) {
                blst::BLST_ERROR::BLST_SUCCESS => {}
                other => {
                    println!("Got error while uncompressing: {other:?}");
                    panic!("Fail to uncompress")
                }
            };
            let mut uncompressed_p1 = blst::blst_p1::default();
            blst::blst_p1_from_affine(&mut uncompressed_p1, &uncompressed_p1_affine);
            assert_eq!(
                uncompressed_p1, p1,
                "result after uncompression must be equal to p1"
            );

            let mut serialized_p1 = [0; 96];
            blst::blst_p1_serialize(serialized_p1.as_mut_ptr(), &p1);
            let mut deserialized_p1_affine = blst::blst_p1_affine::default();
            match blst::blst_p1_deserialize(&mut deserialized_p1_affine, serialized_p1.as_ptr()) {
                blst::BLST_ERROR::BLST_SUCCESS => {}
                other => {
                    println!("Got error while deserializing: {other:?}",);
                    panic!("Fail to deserialize")
                }
            };

            let mut deserialized_p1 = blst::blst_p1::default();
            blst::blst_p1_from_affine(&mut deserialized_p1, &deserialized_p1_affine);
            assert_eq!(
                deserialized_p1, p1,
                "result after deserialization must be equal to p1"
            );
        }
    }

    fn bilinear_map(p1: &blst::blst_p1, p2: &blst::blst_p2) -> blst::blst_fp12 {
        let mut p1_affine = blst::blst_p1_affine::default();
        unsafe {
            blst::blst_p1_to_affine(&mut p1_affine, p1);
        };
        let mut p2_affine = blst::blst_p2_affine::default();
        unsafe {
            blst::blst_p2_to_affine(&mut p2_affine, p2);
        };

        let mut res = blst::blst_fp12::default();
        unsafe {
            blst::blst_miller_loop(&mut res, &p2_affine, &p1_affine);
            blst::blst_final_exp(&mut res, &res);
        };
        res
    }

    /// Computes a - b
    fn blst_p1_sub(a: &blst::blst_p1, b: &blst::blst_p1) -> blst::blst_p1 {
        let mut neg_b = *b;
        unsafe {
            blst::blst_p1_cneg(&mut neg_b, true);
        };
        let mut out = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_add_or_double(&mut out, a, &neg_b);
        };
        out
    }

    /// Computes a - b
    fn blst_p2_sub(a: &blst::blst_p2, b: &blst::blst_p2) -> blst::blst_p2 {
        let mut neg_b = *b;
        unsafe {
            blst::blst_p2_cneg(&mut neg_b, true);
        };
        let mut out = blst::blst_p2::default();
        unsafe {
            blst::blst_p2_add_or_double(&mut out, a, &neg_b);
        };
        out
    }

    fn blst_scalar_from_u8(a: u8) -> blst::blst_scalar {
        let mut le_bytes = [0; 48];
        le_bytes[0] = a;
        let mut scalar = blst::blst_scalar::default();
        unsafe { blst::blst_scalar_from_le_bytes(&mut scalar, le_bytes.as_ptr(), le_bytes.len()) };
        scalar
    }

    #[test]
    fn test_commitment_for_polynomial_degree_one() {
        let mut s_bytes = [0; 48]; // Field elements are encoded in big endian form with 48 bytes
        rand::rng().fill_bytes(&mut s_bytes);
        let setup_artifacts: Vec<_> = SetupArtifactsGenerator::new(s_bytes).take(2).collect();

        // Polynomial to commit is `p(x) = 5x + 10
        // a1 = 5, a0 = 10`
        let a0 = blst_scalar_from_u8(10);
        let mut constant_part = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(
                &mut constant_part,
                blst::blst_p1_generator(),
                a0.b.as_ptr(),
                a0.b.len() * 8,
            );
        };

        let a1 = blst_scalar_from_u8(5);
        let mut order_one_part = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(
                &mut order_one_part,
                &setup_artifacts[1].g1,
                a1.b.as_ptr(),
                a1.b.len() * 8,
            );
        };
        let mut commitment = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_add_or_double(&mut commitment, &constant_part, &order_one_part);
        };

        // We evaluate the polynomial at z = 1: `p(z) = y = p(1) = 15`
        // Quotient polynomial: `q(x) = (p(x) - y) / (x - z) = (5x - 5) / (x - 1) = 5`
        let q_as_scalar = blst_scalar_from_u8(5);
        let mut q_at_s = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(
                &mut q_at_s,
                blst::blst_p1_generator(),
                q_as_scalar.b.as_ptr(),
                q_as_scalar.b.len() * 8,
            );
        };

        let z = unsafe { *blst::blst_p2_generator() };
        let divider = blst_p2_sub(&setup_artifacts[1].g2, &z);
        let lhs = bilinear_map(&q_at_s, &divider);

        let y_as_scalar = blst_scalar_from_u8(15);
        let mut y = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(
                &mut y,
                blst::blst_p1_generator(),
                y_as_scalar.b.as_ptr(),
                y_as_scalar.b.len() * 8,
            );
        };
        let commitment_part = blst_p1_sub(&commitment, &y);
        let g2 = unsafe { *blst::blst_p2_generator() };
        let rhs = bilinear_map(&commitment_part, &g2);

        assert_eq!(lhs, rhs);
    }

    #[test]
    fn test_commitment_for_polynomial_degree_two() {
        let mut s_bytes = [0; 48]; // Field elements are encoded in big endian form with 48 bytes
        rand::rng().fill_bytes(&mut s_bytes);
        let setup_artifacts: Vec<_> = SetupArtifactsGenerator::new(s_bytes).take(3).collect();

        // Polynomial to commit is `p(x) = 2x^2 + 3x + 4`
        // a2 = 2, a1 = 3, a0 = 4
        let a0 = blst_scalar_from_u8(4);
        let mut constant_part = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(
                &mut constant_part,
                blst::blst_p1_generator(),
                a0.b.as_ptr(),
                a0.b.len() * 8,
            );
        };
        let a1 = blst_scalar_from_u8(3);
        let mut order_one_part = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(
                &mut order_one_part,
                &setup_artifacts[1].g1,
                a1.b.as_ptr(),
                a1.b.len() * 8,
            );
        };
        let a2 = blst_scalar_from_u8(2);
        let mut order_two_part = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(
                &mut order_two_part,
                &setup_artifacts[2].g1,
                a2.b.as_ptr(),
                a2.b.len() * 8,
            );
        };
        let mut commitment = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_add_or_double(&mut commitment, &constant_part, &order_one_part);
            blst::blst_p1_add_or_double(&mut commitment, &commitment, &order_two_part);
        };

        // We evaluate the polynomial at z = 2: `p(z) = y = p(2) = 8 + 6 + 4 = 18`
        // Quotient polynomial: `q(x) = (p(x) - y) / (x - z) = (2x^2 + 3x - 14) / (x - 2) = (x - 2) * (2x + 7) / (x - 2) = 2x + 7`
        // b1 = 2, b0 = 7
        let b0 = blst_scalar_from_u8(7);
        let mut q_at_s_constant_part = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(
                &mut q_at_s_constant_part,
                blst::blst_p1_generator(),
                b0.b.as_ptr(),
                b0.b.len() * 8,
            );
        };
        let b1 = blst_scalar_from_u8(2);
        let mut q_at_s_order_one_part = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(
                &mut q_at_s_order_one_part,
                &setup_artifacts[1].g1,
                b1.b.as_ptr(),
                b1.b.len() * 8,
            );
        };
        let mut q_at_s = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_add_or_double(&mut q_at_s, &q_at_s_constant_part, &q_at_s_order_one_part);
        }

        let z_as_scalar = blst_scalar_from_u8(2);
        let mut z = blst::blst_p2::default();
        unsafe {
            blst::blst_p2_mult(
                &mut z,
                blst::blst_p2_generator(),
                z_as_scalar.b.as_ptr(),
                z_as_scalar.b.len() * 8,
            );
        }
        let divider = blst_p2_sub(&setup_artifacts[1].g2, &z);
        let lhs = bilinear_map(&q_at_s, &divider);

        let y_as_scalar = blst_scalar_from_u8(18);
        let mut y = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(
                &mut y,
                blst::blst_p1_generator(),
                y_as_scalar.b.as_ptr(),
                y_as_scalar.b.len() * 8,
            );
        };
        let commitment_part = blst_p1_sub(&commitment, &y);
        let g2 = unsafe { *blst::blst_p2_generator() };
        let rhs = bilinear_map(&commitment_part, &g2);

        assert_eq!(lhs, rhs);
    }
}
