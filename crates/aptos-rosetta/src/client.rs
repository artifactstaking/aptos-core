// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::common::native_coin;
use crate::types::{
    AccountBalanceRequest, AccountBalanceResponse, AccountIdentifier, Amount, BlockRequest,
    BlockResponse, ConstructionCombineRequest, ConstructionCombineResponse,
    ConstructionDeriveRequest, ConstructionDeriveResponse, ConstructionHashRequest,
    ConstructionMetadata, ConstructionMetadataRequest, ConstructionMetadataResponse,
    ConstructionParseRequest, ConstructionParseResponse, ConstructionPayloadsRequest,
    ConstructionPayloadsResponse, ConstructionPreprocessRequest, ConstructionPreprocessResponse,
    ConstructionSubmitRequest, ConstructionSubmitResponse, Error, MetadataRequest,
    NetworkIdentifier, NetworkListResponse, NetworkOptionsResponse, NetworkRequest,
    NetworkStatusResponse, Operation, PreprocessMetadata, PublicKey, Signature, SignatureType,
    TransactionIdentifier, TransactionIdentifierResponse,
};
use anyhow::anyhow;
use aptos_crypto::ed25519::Ed25519PrivateKey;
use aptos_crypto::SigningKey;
use aptos_crypto::{PrivateKey, ValidCryptoMaterialStringExt};
use aptos_rest_client::aptos_api_types::mime_types::JSON;
use aptos_types::account_address::AccountAddress;
use aptos_types::transaction::RawTransaction;
use reqwest::{header::CONTENT_TYPE, Client as ReqwestClient};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use url::Url;

/// Client for testing & interacting with a Rosetta service
#[derive(Debug, Clone)]
pub struct RosettaClient {
    address: Url,
    inner: ReqwestClient,
}

impl RosettaClient {
    pub fn new(address: Url) -> RosettaClient {
        RosettaClient {
            address,
            inner: ReqwestClient::new(),
        }
    }

    pub async fn account_balance(
        &self,
        request: &AccountBalanceRequest,
    ) -> anyhow::Result<AccountBalanceResponse> {
        self.make_call("account/balance", request).await
    }

    pub async fn block(&self, request: &BlockRequest) -> anyhow::Result<BlockResponse> {
        self.make_call("block", request).await
    }

    pub async fn combine(
        &self,
        request: &ConstructionCombineRequest,
    ) -> anyhow::Result<ConstructionCombineResponse> {
        self.make_call("construction/combine", request).await
    }

    pub async fn derive(
        &self,
        request: &ConstructionDeriveRequest,
    ) -> anyhow::Result<ConstructionDeriveResponse> {
        self.make_call("construction/derive", request).await
    }

    pub async fn hash(
        &self,
        request: &ConstructionHashRequest,
    ) -> anyhow::Result<TransactionIdentifierResponse> {
        self.make_call("construction/hash", request).await
    }

    pub async fn metadata(
        &self,
        request: &ConstructionMetadataRequest,
    ) -> anyhow::Result<ConstructionMetadataResponse> {
        self.make_call("construction/metadata", request).await
    }

    pub async fn parse(
        &self,
        request: &ConstructionParseRequest,
    ) -> anyhow::Result<ConstructionParseResponse> {
        self.make_call("construction/parse", request).await
    }

    pub async fn payloads(
        &self,
        request: &ConstructionPayloadsRequest,
    ) -> anyhow::Result<ConstructionPayloadsResponse> {
        self.make_call("construction/payloads", request).await
    }

    pub async fn preprocess(
        &self,
        request: &ConstructionPreprocessRequest,
    ) -> anyhow::Result<ConstructionPreprocessResponse> {
        self.make_call("construction/preprocess", request).await
    }

    pub async fn submit(
        &self,
        request: &ConstructionSubmitRequest,
    ) -> anyhow::Result<ConstructionSubmitResponse> {
        self.make_call("construction/submit", request).await
    }

    pub async fn network_list(&self) -> anyhow::Result<NetworkListResponse> {
        self.make_call("network/list", &MetadataRequest {}).await
    }

    pub async fn network_options(
        &self,
        request: &NetworkRequest,
    ) -> anyhow::Result<NetworkOptionsResponse> {
        self.make_call("network/options", request).await
    }

    pub async fn network_status(
        &self,
        request: &NetworkRequest,
    ) -> anyhow::Result<NetworkStatusResponse> {
        self.make_call("network/status", request).await
    }

    async fn make_call<'a, I: Serialize + Debug, O: DeserializeOwned>(
        &'a self,
        path: &'static str,
        request: &'a I,
    ) -> anyhow::Result<O> {
        let response = self
            .inner
            .post(self.address.join(path)?)
            .header(CONTENT_TYPE, JSON)
            .body(serde_json::to_string(request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: Error = response.json().await?;
            return Err(anyhow!("Failed API with: {:?}", error));
        }

        Ok(response.json().await?)
    }

    pub async fn create_account(
        &self,
        network_identifier: &NetworkIdentifier,
        private_key: &Ed25519PrivateKey,
        new_account: AccountAddress,
        expiry_time_secs: u64,
        sequence_number: Option<u64>,
    ) -> anyhow::Result<TransactionIdentifier> {
        let sender = self
            .get_account_address(network_identifier.clone(), private_key)
            .await?;
        let mut keys = HashMap::new();
        keys.insert(sender, private_key);

        // A create account transaction is just a Create account operation
        let operations = vec![Operation::create_account(0, None, new_account, sender)];

        self.submit_operations(
            network_identifier.clone(),
            &keys,
            operations,
            expiry_time_secs,
            sequence_number,
        )
        .await
    }

    pub async fn transfer(
        &self,
        network_identifier: &NetworkIdentifier,
        private_key: &Ed25519PrivateKey,
        receiver: AccountAddress,
        amount: u64,
        expiry_time_secs: u64,
        sequence_number: Option<u64>,
    ) -> anyhow::Result<TransactionIdentifier> {
        let sender = self
            .get_account_address(network_identifier.clone(), private_key)
            .await?;
        let mut keys = HashMap::new();
        keys.insert(sender, private_key);

        // A transfer operation is made up of a withdraw and a deposit
        let operations = vec![
            Operation::withdraw(0, None, sender, native_coin(), amount),
            Operation::deposit(1, None, receiver, native_coin(), amount),
        ];

        self.submit_operations(
            network_identifier.clone(),
            &keys,
            operations,
            expiry_time_secs,
            sequence_number,
        )
        .await
    }

    /// Retrieves the account address from the derivation path if there isn't an overriding account specified
    async fn get_account_address(
        &self,
        network_identifier: NetworkIdentifier,
        private_key: &Ed25519PrivateKey,
    ) -> anyhow::Result<AccountAddress> {
        Ok(self
            .derive_account(network_identifier, private_key.public_key().try_into()?)
            .await?
            .account_address()?)
    }

    /// Submits the operations to the blockchain
    async fn submit_operations(
        &self,
        network_identifier: NetworkIdentifier,
        keys: &HashMap<AccountAddress, &Ed25519PrivateKey>,
        operations: Vec<Operation>,
        expiry_time_secs: u64,
        sequence_number: Option<u64>,
    ) -> anyhow::Result<TransactionIdentifier> {
        // Retrieve txn metadata
        let (metadata, public_keys) = self
            .metadata_for_ops(
                network_identifier.clone(),
                operations.clone(),
                10000,
                1,
                expiry_time_secs,
                sequence_number,
                keys,
            )
            .await?;

        // Build the transaction, sign it, and submit it
        let response = self
            .unsigned_transaction(
                network_identifier.clone(),
                operations.clone(),
                metadata.metadata,
                public_keys,
            )
            .await?;
        let signed_txn = self
            .sign_transaction(network_identifier.clone(), keys, response, operations)
            .await?;
        self.submit_transaction(network_identifier, signed_txn)
            .await
    }

    /// Derives an [`AccountAddress`] from the [`PublicKey`]
    async fn derive_account(
        &self,
        network_identifier: NetworkIdentifier,
        public_key: PublicKey,
    ) -> anyhow::Result<AccountIdentifier> {
        if let ConstructionDeriveResponse {
            account_identifier: Some(account_id),
        } = self
            .derive(&ConstructionDeriveRequest {
                network_identifier,
                public_key,
            })
            .await?
        {
            Ok(account_id)
        } else {
            return Err(anyhow!("Failed to find account address for key"));
        }
    }

    /// Retrieves the metadata for the set of operations
    async fn metadata_for_ops(
        &self,
        network_identifier: NetworkIdentifier,
        operations: Vec<Operation>,
        max_fee: u64,
        fee_multiplier: u32,
        expiry_time_secs: u64,
        sequence_number: Option<u64>,
        keys: &HashMap<AccountAddress, &Ed25519PrivateKey>,
    ) -> anyhow::Result<(ConstructionMetadataResponse, Vec<PublicKey>)> {
        // Request the given operation with the given gas constraints
        let amount = val_to_amount(max_fee, false);
        let preprocess_response = self
            .preprocess(&ConstructionPreprocessRequest {
                network_identifier: network_identifier.clone(),
                operations,
                max_fee: Some(vec![amount]),
                suggested_fee_multiplier: Some(fee_multiplier as f64),
                metadata: Some(PreprocessMetadata {
                    expiry_time_secs: Some(expiry_time_secs),
                    sequence_number,
                }),
            })
            .await?;

        // Process the required public keys
        let mut public_keys = Vec::new();
        if let Some(accounts) = preprocess_response.required_public_keys {
            for account in accounts {
                if let Some(key) = keys.get(&account.account_address()?) {
                    public_keys.push(key.public_key().try_into()?);
                } else {
                    return Err(anyhow!("No public key found for account"));
                }
            }
        } else {
            return Err(anyhow!("No public keys found required for transaction"));
        };

        // Request the metadata
        if let Some(options) = preprocess_response.options {
            self.metadata(&ConstructionMetadataRequest {
                network_identifier,
                options,
                public_keys: public_keys.clone(),
            })
            .await
            .map(|response| (response, public_keys))
        } else {
            Err(anyhow!(
                "No metadata options returned from preprocess response"
            ))
        }
    }

    /// Build an unsigned transaction
    async fn unsigned_transaction(
        &self,
        network_identifier: NetworkIdentifier,
        operations: Vec<Operation>,
        metadata: ConstructionMetadata,
        public_keys: Vec<PublicKey>,
    ) -> anyhow::Result<ConstructionPayloadsResponse> {
        // Build the unsigned transaction
        let payloads = self
            .payloads(&ConstructionPayloadsRequest {
                network_identifier: network_identifier.clone(),
                operations: operations.clone(),
                metadata: Some(metadata),
                public_keys: Some(public_keys),
            })
            .await?;

        // Verify that we can parse the transaction
        let response = self
            .parse(&ConstructionParseRequest {
                network_identifier,
                signed: false,
                transaction: payloads.unsigned_transaction.clone(),
            })
            .await?;

        if response.account_identifier_signers.is_some() {
            Err(anyhow!("Signers were in the unsigned transaction!"))
        } else if operations != response.operations {
            Err(anyhow!(
                "Operations were not parsed to be the same as input! Expected {:?} Got {:?}",
                operations,
                response.operations
            ))
        } else {
            Ok(payloads)
        }
    }

    /// Signs a transaction and combines it with an unsigned transaction
    async fn sign_transaction(
        &self,
        network_identifier: NetworkIdentifier,
        keys: &HashMap<AccountAddress, &Ed25519PrivateKey>,
        unsigned_response: ConstructionPayloadsResponse,
        operations: Vec<Operation>,
    ) -> anyhow::Result<String> {
        let mut signatures = Vec::new();
        let mut signers: Vec<AccountIdentifier> = Vec::new();

        // Sign the unsigned transaction
        let unsigned_transaction: RawTransaction = bcs::from_bytes(&hex::decode(
            unsigned_response.unsigned_transaction.clone(),
        )?)?;
        let signing_message = hex::encode(unsigned_transaction.signing_message());

        // Sign the payload if it matches the unsigned transaction
        for payload in unsigned_response.payloads.into_iter() {
            let account = payload
                .account_identifier
                .as_ref()
                .expect("Must have an account");
            let private_key = keys
                .get(&account.account_address()?)
                .expect("Should have a private key");
            signers.push(account.clone());

            assert_eq!(signing_message, payload.hex_bytes);
            let txn_signature = private_key.sign(&unsigned_transaction);
            signatures.push(Signature {
                signing_payload: payload,
                public_key: private_key.public_key().try_into()?,
                signature_type: SignatureType::Ed25519,
                hex_bytes: txn_signature.to_encoded_string()?,
            });
        }

        // Build the signed transaction
        let signed_response = self
            .combine(&ConstructionCombineRequest {
                network_identifier: network_identifier.clone(),
                unsigned_transaction: unsigned_response.unsigned_transaction,
                signatures,
            })
            .await?;

        // Verify transaction can be parsed properly
        let response = self
            .parse(&ConstructionParseRequest {
                network_identifier,
                signed: true,
                transaction: signed_response.signed_transaction.clone(),
            })
            .await?;

        // Signers must match exactly
        if let Some(parsed_signers) = response.account_identifier_signers {
            if signers != parsed_signers {
                return Err(anyhow!(
                    "Signers don't match Expected: {:?} Got: {:?}",
                    signers,
                    parsed_signers
                ));
            }
        } else {
            return Err(anyhow!("Signers were in the unsigned transaction!"));
        }

        // Operations must match exactly
        if operations != response.operations {
            Err(anyhow!(
                "Operations were not parsed to be the same as input! Expected {:?} Got {:?}",
                operations,
                response.operations
            ))
        } else {
            Ok(signed_response.signed_transaction)
        }
    }

    /// Submit a transaction to the blockchain
    async fn submit_transaction(
        &self,
        network_identifier: NetworkIdentifier,
        signed_transaction: String,
    ) -> anyhow::Result<TransactionIdentifier> {
        Ok(self
            .submit(&ConstructionSubmitRequest {
                network_identifier,
                signed_transaction,
            })
            .await?
            .transaction_identifier)
    }
}

/// Converts a value to a Rosetta [`Amount`]
///
/// Only works with the native coin
fn val_to_amount(amount: u64, withdraw: bool) -> Amount {
    let value = if withdraw {
        format!("-{}", amount)
    } else {
        amount.to_string()
    };
    Amount {
        value,
        currency: native_coin(),
    }
}
