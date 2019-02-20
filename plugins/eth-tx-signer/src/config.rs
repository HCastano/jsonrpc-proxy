/// Configuration options of an upstream
#[derive(Clone, Debug)]
pub enum Param {}

/// Returns all configuration parameters for WS upstream.
pub fn params() -> Vec<cli_params::Param<Param>> {
    unimplemented!();
}
