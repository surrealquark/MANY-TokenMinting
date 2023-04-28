use many_client::client::{Client, Response};
use many_modules::module::{Module, ModuleResult};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_cbor::Value;
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenMintingRequest {
    amount: u64,
    recipient: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenMintingResponse {
    tx_hash: String,
}

#[derive(Debug, Error)]
pub enum TokenMintingError {
    #[error("Failed to serialize token minting request: {0}")]
    SerdeError(#[from] serde_cbor::Error),

    #[error("Failed to broadcast transaction: {0}")]
    BroadcastError(#[from] anyhow::Error),
}

pub struct TokenMintingModule {
    client: Client,
}

impl TokenMintingModule {
    pub fn new(client: Client) -> TokenMintingModule {
        TokenMintingModule { client }
    }

    pub fn mint_tokens(&self, request: TokenMintingRequest) -> Result<TokenMintingResponse, TokenMintingError> {
        let tx_hash = format!("{:x}", rand::thread_rng().gen::<u128>());

        let tx = json!({
            "type": "token_minting",
            "data": {
                "amount": request.amount,
                "recipient": request.recipient,
                "tx_hash": tx_hash.clone(),
            }
        });

        let response = self.client.broadcast_tx_sync(tx.to_string())
            .map_err(TokenMintingError::BroadcastError)?;

        Ok(TokenMintingResponse { tx_hash })
    }
}

impl Module for TokenMintingModule {
    fn handle(&self, request: Value) -> ModuleResult<Response> {
        let token_minting_request: TokenMintingRequest = serde_cbor::from_value(request)?;
        let token_minting_response = self.mint_tokens(token_minting_request)?;

        let response = json!({
            "result": {
                "tx_hash": token_minting_response.tx_hash,
            }
        });

        Ok(Response::Value(response.into()))
    }
}