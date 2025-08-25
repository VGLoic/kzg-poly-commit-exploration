# KZG Polynomial Commitment Exploration

KZG polynomial commitment exploration is a Rust CLI application for learning and experimenting with KZG (Kate, Zaverucha and Goldberg) polynomial commitments. The application provides commands for trusted setup, polynomial commitment, evaluation, and verification using the BLS12-381 elliptic curve.

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

## Working Effectively

### Bootstrap and Build
- Install Rust if not available: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Source environment: `source ~/.cargo/env`
- Check compilation: `cargo check` -- takes ~20 seconds on first run. NEVER CANCEL. Set timeout to 180+ seconds.
- Debug build: `cargo build` -- takes ~7 seconds after dependencies downloaded. NEVER CANCEL. Set timeout to 300+ seconds.
- Release build: `cargo build --release` -- takes ~18 seconds. NEVER CANCEL. Set timeout to 600+ seconds.

### Testing
- Run all tests: `cargo test` -- takes ~3 seconds. NEVER CANCEL. Set timeout to 300+ seconds.
- Tests include unit tests for curves, scalars, polynomials, and Kate proof generation/verification.

### Code Quality
- Format code: `cargo fmt`
- Check formatting: `cargo fmt --check` -- instant execution.
- Run linter: `cargo clippy -- -Dwarnings` -- takes ~1 second. NEVER CANCEL. Set timeout to 120+ seconds.

### Environment Configuration
- Copy environment template: `cp .env.example .env`
- Configure logging: Add `LOG_LEVEL=debug` to `.env` for detailed output.
- Default log levels: no debug flag = INFO, `-d` = DEBUG, `-dd` = TRACE.

## Application Usage

### CLI Commands
- Show help: `cargo run -- --help`
- Debug mode: `cargo run -- -d <command>` (or `-dd` for trace level)

### Complete Workflow
1. **Trusted Setup**: `cargo run -- trusted-setup`
   - Generates cryptographic setup artifacts in `./artifacts/setup.json`
   - Creates random secret and computes powers on elliptic curves
   - Takes ~1 second. Creates `./artifacts/` directory if missing.

2. **Polynomial Commitment**: `cargo run -- commit <coefficients>`
   - Example: `cargo run -- commit 1 2 3` commits to polynomial `3x² + 2x + 1`
   - Coefficients are in ascending degree order (constant, x, x², etc.)
   - Maximum degree supported: 9 (so max 10 coefficients)
   - Creates `./artifacts/commitment.json`
   - Requires setup artifacts from step 1

3. **Polynomial Evaluation**: `cargo run -- evaluate <x>`
   - Example: `cargo run -- evaluate 5` evaluates polynomial at x=5
   - Generates Kate proof for the evaluation
   - Creates `./artifacts/evaluation.json`
   - Requires both setup and commitment artifacts

4. **Verification**: `cargo run -- verify-evaluation`
   - Verifies the previous evaluation and its proof
   - Requires all three artifact files
   - Returns success/failure of proof verification

## Validation

### Manual Testing Scenarios
Always run through at least one complete end-to-end scenario after making changes:

1. **Basic Workflow Test**:
   ```bash
   # Clean artifacts
   rm -rf ./artifacts
   
   # Run complete workflow
   cargo run -- trusted-setup
   cargo run -- commit 1 2 3
   cargo run -- evaluate 5
   cargo run -- verify-evaluation
   ```
   Expected: All commands succeed, P(5) = 86 for polynomial 3x² + 2x + 1

2. **Maximum Degree Test**:
   ```bash
   cargo run -- trusted-setup
   cargo run -- commit 1 2 3 4 5 6 7 8 9 10
   cargo run -- evaluate 2
   cargo run -- verify-evaluation
   ```
   Expected: All commands succeed with degree-9 polynomial

3. **Error Handling Test**:
   ```bash
   cargo run -- commit 1 2 3 4 5 6 7 8 9 10 11
   ```
   Expected: Error about degree limit exceeded

### CI Validation
Always run these before committing changes (matches .github/workflows/ci.yml):
- `cargo fmt --check` -- format validation
- `cargo test` -- all tests pass
- `cargo build --verbose` -- successful build
- `cargo clippy -- -Dwarnings` -- no linting warnings

## Code Structure

### Key Modules
- `src/main.rs` - CLI interface and command handling
- `src/lib.rs` - Library entry point with integration tests
- `src/curves.rs` - Elliptic curve operations (G1, G2 points)
- `src/scalar.rs` - Finite field scalar operations
- `src/polynomial.rs` - Polynomial operations and commitment
- `src/trusted_setup.rs` - Setup ceremony artifact generation

### Important Constants
- `MAX_DEGREE: u32 = 9` - Maximum polynomial degree supported
- Artifact paths: `./artifacts/setup.json`, `./artifacts/commitment.json`, `./artifacts/evaluation.json`

### Dependencies
- `blst` - BLS12-381 elliptic curve implementation
- `clap` - CLI argument parsing
- `serde`/`serde_json` - Serialization for artifacts
- `anyhow` - Error handling
- `dotenvy` - Environment variable loading

## Common Tasks

### Repository Root Structure
```
.
├── .env.example          # Environment variable template  
├── .github/
│   └── workflows/
│       └── ci.yml        # CI pipeline (format, test, build, clippy)
├── .gitignore           # Excludes /target, .env, /artifacts, .vscode
├── Cargo.toml           # Rust package configuration
├── Cargo.lock           # Dependency lock file
├── LICENSE              # Project license
├── README.md            # Detailed project documentation
└── src/                 # Source code directory
```

### Artifact Files
After running commands, artifacts are stored in:
- `./artifacts/setup.json` - Trusted setup points (created by trusted-setup)
- `./artifacts/commitment.json` - Polynomial and commitment (created by commit)  
- `./artifacts/evaluation.json` - Evaluation and proof (created by evaluate)

### Development Notes
- The application uses BLS12-381 elliptic curve for cryptographic operations
- Polynomials are represented with coefficients in ascending degree order
- All artifacts are JSON serialized for human readability
- Setup uses cryptographically secure randomness (not for production use)
- Commands must be run in sequence: setup → commit → evaluate → verify

### Troubleshooting
- **"Trusted setup artifacts have not been found"**: Run `cargo run -- trusted-setup` first
- **"Commitment artifact has not been found"**: Run `cargo run -- commit <coefficients>` first  
- **Degree too high error**: Use maximum 10 coefficients (degree 9)
- **Build failures**: Ensure Rust toolchain is installed and up to date

### Performance Expectations
- **NEVER CANCEL**: All build and test commands should complete. Set generous timeouts.
- First build: ~20 seconds (dependency download)
- Subsequent builds: ~7 seconds (debug), ~18 seconds (release)
- Tests: ~3 seconds for full suite
- CLI commands: <1 second each
- Format/lint checks: <2 seconds each