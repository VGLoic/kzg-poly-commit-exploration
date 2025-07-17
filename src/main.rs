use clap::{Parser, Subcommand};
use std::str::FromStr;

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
    /// Print "Hello, world!"
    HelloWorld {},
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
        Some(Commands::HelloWorld {}) => {
            log::info!("Hello, world!");
        }
        None => {
            log::warn!("No command has been input")
        }
    }
}

#[cfg(test)]
mod tests {
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
}
