//! TODO: Add description here

// Here's the gameplan:
// 1. Check the incoming request and make sure it is eth_sendTransaction
// 2. Send to function to check validity of the fields
//    Optional, may just want to fill in some of the missing fields ourselves via
//    more RPC requests to the node (nonce, gas price, etc)
// 3. Once we have all the required fields, will want to get the transaction signed
//    This seems like a bit of a PITA
// 4. Transform it into an eth_sendRawTransaction request
// 5. Pass this to the `upstream` middleware for processing


#[warn(missing_docs)]
#[warn(unused_extern_crates)]

use ethereum_types::{Address, U256};
// use ethsign::Signature;
use rpc;
use rpc::types::Error;
use rpc::{
    futures::Future,
    futures::future::{self, Either},
};
use serde::Deserialize;
use upstream::helpers;

pub mod config;

// Maybe I should do something like this....
// enum Transaction {
//     Signed(TransactionData),
//     Unsigned(TransactionData),
// }

#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
// May need "condition" field here too
struct TransactionParams {
    from: Address,
    to: Address,
    gas: U256,
    gas_price: U256,
    value: U256,
    data: Vec<u8>,
    nonce: U256,
}

// In Parity-Eth's CallRequest all these are wrapped in Options
#[derive(Debug, Default, PartialEq, Deserialize)]
struct Transaction {
    from: Option<Address>,
    to: Option<Address>,
    gas: Option<U256>,
    gas_price: Option<U256>,
    value: Option<U256>,
    data: Option<Vec<u8>>,
    nonce: Option<U256>,
}

// Transaction { from, maybeTo, data, value, gasPrice, gas, nonce }
impl Transaction {
    fn new(params: &TransactionParams) -> Self {
        Self {
            from: Some(params.from),
            to: Some(params.to),
            gas: Some(params.gas),
            gas_price: Some(params.gas_price),
            value: Some(params.value),
            data: Some(params.data.to_owned()),
            nonce: Some(params.nonce),
        }
    }
}

#[derive(Debug)]
struct EthTxSigner {
    transaction: Transaction,
}

impl EthTxSigner {
    fn new() -> Self {
        Self { transaction: Transaction::default() }
    }

    fn get_signed_request(&self, request: &rpc::Call) -> Result<Transaction, rpc::error::Error> {
        let params: TransactionParams = get_request_params(request)?;
        dbg!(&params);

        // TODO: Will want to handle the Err case by trying to request common fields
        // like nonce and gas price from the node. Ignoring for now.

        let tx_request = Transaction::new(&params);

        Ok(tx_request)
    }

    fn _request_missing_fields(&self, _request: rpc::Call) {
        unimplemented!()
    }

    fn _sign_transaction() -> Result<Transaction, ()> {
        unimplemented!()
    }
}


/// Middleware for remotely signing ETH transactions
#[derive(Debug)]
pub struct Middleware {
    signer: EthTxSigner,
    // TODO: Make this a std::path::Path
    key_file: String,
}

impl Middleware {
    /// Creates new remote Eth signer middleware
    pub fn new(_params: &[config::Param]) -> Self {
        // TODO: Will want a config.rs file just like in the Permissioning crate
        Middleware {
            signer: EthTxSigner::new(),
            key_file: ".".into(),
        }
    }
}

impl<M: rpc::Metadata> rpc::Middleware<M> for Middleware {
    type Future = rpc::middleware::NoopFuture;
    type CallFuture = rpc::futures::future::FutureResult<Option<rpc::Output>, ()>;

    fn on_call<F, X>(&self, request: rpc::Call, meta: M, next: F) -> Either<Self::CallFuture, X> where
        F: FnOnce(rpc::Call, M) -> X + Send,
        X: Future<Item = Option<rpc::Output>, Error = ()> + Send + 'static,
    {
        dbg!(&request);

        if !is_send_tx_request(&request) {
            // TODO: Return error complaining about wrong call
            return Either::B(next(request, meta));
        }

        let signed_request = self.signer.get_signed_request(&request);
        dbg!(signed_request);

        // TODO: Get a proper return type here
        Either::B(next(request, meta))
    }
}


fn is_send_tx_request(request: &rpc::Call) -> bool {
    let method = helpers::get_method_name(request);
    match method {
        Some(name) => match name {
            "eth_sendTransaction" => true,
            _ => false,
        },
        None => false,
    }
}

fn get_request_params(request: &rpc::Call) -> Result<TransactionParams, rpc::types::Error> {
    let params = match request {
        rpc::Call::MethodCall(rpc::MethodCall { ref params, .. }) => {
            params.clone()
        },
        _ => return Err(Error::invalid_request()),
    };

    dbg!(&params);
    // Need to parse this into a tuple because Ethereum calls
    // receive their parameters inside an array
    let (tx_params, ): (TransactionParams, ) = params.parse()?;
    Ok(tx_params)
}
