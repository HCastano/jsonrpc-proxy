//! CLI configuration for Ethereum signer.

#[derive(Clone, Debug)]
pub enum Param {
    /// Path to JSON keystore file
    Config(String),
}

/// Returns all configuration parameters for Eth signer.
pub fn params() -> Vec<cli_params::Param<Param>> {
    vec![
        cli_params::Param::new(
            "Ethereum Transaction Signer",
            "eth-tx-signer-config",
            "A path to a JSON file containing keys.",
            "-",
            |path: String| {
                if &path == "-" {
                    // Will want to default to this directory
                    // Or maybe the std. Parity Eth keystore?
                    return Ok(Param::Config(".".into()))
                }

                // Just want to return the path to the file
                // Will deal with parsing the file later (during the
                // signing step)
                Ok(Param::Config(path))
            }
        )
    ]
}
