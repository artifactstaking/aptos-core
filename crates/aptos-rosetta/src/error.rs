// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::{types, types::ErrorDetails};
use aptos_rest_client::aptos_api_types::AptosErrorCode;
use aptos_rest_client::error::RestError;
use hex::FromHexError;
use move_deps::move_core_types::account_address::AccountAddressParseError;
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;
use warp::{http::StatusCode, reply::Reply};

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Deserialize, Serialize)]
pub enum ApiError {
    BlockParameterConflict,
    TransactionIsPending,
    NetworkIdentifierMismatch,
    ChainIdMismatch,
    DeserializationFailed(Option<String>),
    InvalidTransferOperations(Option<&'static str>),
    InvalidSignatureType,
    InvalidMaxGasFees,
    InvalidGasMultiplier,
    InvalidOperations,
    MissingPayloadMetadata,
    UnsupportedCurrency(Option<String>),
    UnsupportedSignatureCount(Option<usize>),
    NodeIsOffline,
    TransactionParseError(Option<String>),
    InternalError(Option<String>),

    // Below here are codes directly from the REST API
    AccountNotFound(Option<String>),
    ResourceNotFound(Option<String>),
    ModuleNotFound(Option<String>),
    StructFieldNotFound(Option<String>),
    VersionNotFound(Option<String>),
    TransactionNotFound(Option<String>),
    TableItemNotFound(Option<String>),
    BlockNotFound(Option<String>),
    VersionPruned(Option<String>),
    BlockPruned(Option<String>),
    InvalidInput(Option<String>),
    InvalidTransactionUpdate(Option<String>),
    SequenceNumberTooOld(Option<String>),
    VmError(Option<String>),
    MempoolIsFull(Option<String>),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ApiError {}

impl ApiError {
    pub fn all() -> Vec<ApiError> {
        use ApiError::*;
        vec![
            BlockParameterConflict,
            TransactionIsPending,
            NetworkIdentifierMismatch,
            ChainIdMismatch,
            DeserializationFailed(None),
            InvalidTransferOperations(None),
            InvalidSignatureType,
            InvalidMaxGasFees,
            InvalidGasMultiplier,
            InvalidOperations,
            MissingPayloadMetadata,
            UnsupportedCurrency(None),
            UnsupportedSignatureCount(None),
            NodeIsOffline,
            TransactionParseError(None),
            InternalError(None),
            AccountNotFound(None),
            ResourceNotFound(None),
            ModuleNotFound(None),
            StructFieldNotFound(None),
            VersionNotFound(None),
            TransactionNotFound(None),
            TableItemNotFound(None),
            BlockNotFound(None),
            VersionPruned(None),
            BlockPruned(None),
            InvalidInput(None),
            InvalidTransactionUpdate(None),
            SequenceNumberTooOld(None),
            VmError(None),
            MempoolIsFull(None),
        ]
    }

    pub fn code(&self) -> u32 {
        use ApiError::*;
        match self {
            BlockParameterConflict => 0,
            TransactionIsPending => 1,
            NetworkIdentifierMismatch => 2,
            ChainIdMismatch => 3,
            DeserializationFailed(_) => 4,
            InvalidTransferOperations(_) => 5,
            InvalidSignatureType => 6,
            InvalidMaxGasFees => 7,
            InvalidGasMultiplier => 8,
            InvalidOperations => 9,
            MissingPayloadMetadata => 10,
            UnsupportedCurrency(_) => 11,
            UnsupportedSignatureCount(_) => 12,
            NodeIsOffline => 13,
            TransactionParseError(_) => 14,
            InternalError(_) => AptosErrorCode::InternalError.as_u32(),
            AccountNotFound(_) => AptosErrorCode::AccountNotFound.as_u32(),
            ResourceNotFound(_) => AptosErrorCode::ResourceNotFound.as_u32(),
            ModuleNotFound(_) => AptosErrorCode::ModuleNotFound.as_u32(),
            StructFieldNotFound(_) => AptosErrorCode::StructFieldNotFound.as_u32(),
            VersionNotFound(_) => AptosErrorCode::VersionNotFound.as_u32(),
            TransactionNotFound(_) => AptosErrorCode::TransactionNotFound.as_u32(),
            TableItemNotFound(_) => AptosErrorCode::TableItemNotFound.as_u32(),
            BlockNotFound(_) => AptosErrorCode::BlockNotFound.as_u32(),
            VersionPruned(_) => AptosErrorCode::VersionPruned.as_u32(),
            BlockPruned(_) => AptosErrorCode::BlockPruned.as_u32(),
            InvalidInput(_) => AptosErrorCode::InvalidInput.as_u32(),
            InvalidTransactionUpdate(_) => AptosErrorCode::InvalidTransactionUpdate.as_u32(),
            SequenceNumberTooOld(_) => AptosErrorCode::SequenceNumberTooOld.as_u32(),
            VmError(_) => AptosErrorCode::VmError.as_u32(),
            MempoolIsFull(_) => AptosErrorCode::MempoolIsFull.as_u32(),
        }
    }

    pub fn retriable(&self) -> bool {
        use ApiError::*;
        matches!(
            self,
            AccountNotFound(_) | BlockNotFound(_) | MempoolIsFull(_)
        )
    }

    pub fn status_code(&self) -> StatusCode {
        use ApiError::*;
        match self {
            AccountNotFound(_)
            | BlockNotFound(_)
            | ResourceNotFound(_)
            | ModuleNotFound(_)
            | VersionNotFound(_)
            | TransactionNotFound(_)
            | StructFieldNotFound(_)
            | TableItemNotFound(_) => StatusCode::NOT_FOUND,
            MempoolIsFull(_) => StatusCode::INSUFFICIENT_STORAGE,
            BlockPruned(_) | VersionPruned(_) => StatusCode::GONE,
            NodeIsOffline => StatusCode::METHOD_NOT_ALLOWED,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ApiError::BlockParameterConflict => {
                "Block parameter conflict. Must provide either hash or index but not both"
            }
            ApiError::TransactionIsPending => "Transaction is pending",
            ApiError::NetworkIdentifierMismatch => "Network identifier doesn't match",
            ApiError::ChainIdMismatch => "Chain Id doesn't match",
            ApiError::DeserializationFailed(_) => "Deserialization failed",
            ApiError::InvalidTransferOperations(_) => "Invalid operations for a transfer",
            ApiError::AccountNotFound(_) => "Account not found",
            ApiError::InvalidSignatureType => "Invalid signature type",
            ApiError::InvalidMaxGasFees => "Invalid max gas fee",
            ApiError::InvalidGasMultiplier => "Invalid gas multiplier",
            ApiError::InvalidOperations => "Invalid operations",
            ApiError::MissingPayloadMetadata => "Payload metadata is missing",
            ApiError::UnsupportedCurrency(_) => "Currency is unsupported",
            ApiError::UnsupportedSignatureCount(_) => "Number of signatures is not supported",
            ApiError::NodeIsOffline => "This API is unavailable for the node because he's offline",
            ApiError::BlockNotFound(_) => "Block is missing events",
            ApiError::TransactionParseError(_) => "Transaction failed to parse",
            ApiError::InternalError(_) => "Internal error",
            ApiError::ResourceNotFound(_) => "Resource not found",
            ApiError::ModuleNotFound(_) => "Module not found",
            ApiError::StructFieldNotFound(_) => "Struct field not found",
            ApiError::VersionNotFound(_) => "Version not found",
            ApiError::TransactionNotFound(_) => "Transaction not found",
            ApiError::TableItemNotFound(_) => "Table item not found",
            ApiError::VersionPruned(_) => "Version pruned",
            ApiError::BlockPruned(_) => "Block pruned",
            ApiError::InvalidInput(_) => "Invalid input",
            ApiError::InvalidTransactionUpdate(_) => "Invalid transaction update.  Can only update gas unit price",
            ApiError::SequenceNumberTooOld(_) => "Sequence number too old.  Please create a new transaction with an updated sequence number",
            ApiError::VmError(_) => "Transaction submission failed due to VM error",
            ApiError::MempoolIsFull(_) => "Mempool is full all accounts",
        }
        .to_string()
    }

    pub fn details(self) -> Option<ErrorDetails> {
        match self {
            ApiError::DeserializationFailed(inner) => inner,
            ApiError::InvalidTransferOperations(inner) => inner.map(|inner| inner.to_string()),
            ApiError::UnsupportedCurrency(inner) => inner,
            ApiError::UnsupportedSignatureCount(inner) => inner.map(|inner| inner.to_string()),
            ApiError::TransactionParseError(inner) => inner,
            ApiError::InternalError(inner) => inner,
            ApiError::AccountNotFound(inner) => inner,
            ApiError::ResourceNotFound(inner) => inner,
            ApiError::ModuleNotFound(inner) => inner,
            ApiError::StructFieldNotFound(inner) => inner,
            ApiError::VersionNotFound(inner) => inner,
            ApiError::TransactionNotFound(inner) => inner,
            ApiError::TableItemNotFound(inner) => inner,
            ApiError::BlockNotFound(inner) => inner,
            ApiError::VersionPruned(inner) => inner,
            ApiError::BlockPruned(inner) => inner,
            ApiError::InvalidInput(inner) => inner,
            ApiError::InvalidTransactionUpdate(inner) => inner,
            ApiError::SequenceNumberTooOld(inner) => inner,
            ApiError::VmError(inner) => inner,
            ApiError::MempoolIsFull(inner) => inner,
            _ => None,
        }
        .map(|details| ErrorDetails { details })
    }

    pub fn deserialization_failed(type_: &str) -> ApiError {
        ApiError::DeserializationFailed(Some(type_.to_string()))
    }

    pub fn into_error(self) -> types::Error {
        self.into()
    }
}

impl From<ApiError> for types::Error {
    fn from(error: ApiError) -> Self {
        let message = error.message();
        let code = error.code();
        let retriable = error.retriable();
        let details = error.details();
        types::Error {
            message,
            code,
            retriable,
            details,
            description: None,
        }
    }
}

impl From<RestError> for ApiError {
    fn from(err: RestError) -> Self {
        match err {
            RestError::Api(err) => match err.error.error_code {
                AptosErrorCode::AccountNotFound => {
                    ApiError::AccountNotFound(Some(err.error.message))
                }
                AptosErrorCode::ResourceNotFound => {
                    ApiError::ResourceNotFound(Some(err.error.message))
                }
                AptosErrorCode::ModuleNotFound => ApiError::ModuleNotFound(Some(err.error.message)),
                AptosErrorCode::StructFieldNotFound => {
                    ApiError::StructFieldNotFound(Some(err.error.message))
                }
                AptosErrorCode::VersionNotFound => {
                    ApiError::VersionNotFound(Some(err.error.message))
                }
                AptosErrorCode::TransactionNotFound => {
                    ApiError::TransactionNotFound(Some(err.error.message))
                }
                AptosErrorCode::TableItemNotFound => {
                    ApiError::TableItemNotFound(Some(err.error.message))
                }
                AptosErrorCode::BlockNotFound => ApiError::BlockNotFound(Some(err.error.message)),
                AptosErrorCode::VersionPruned => ApiError::VersionPruned(Some(err.error.message)),
                AptosErrorCode::BlockPruned => ApiError::BlockPruned(Some(err.error.message)),
                AptosErrorCode::InvalidInput => ApiError::InvalidInput(Some(err.error.message)),
                AptosErrorCode::InvalidTransactionUpdate => {
                    ApiError::InvalidInput(Some(err.error.message))
                }
                AptosErrorCode::SequenceNumberTooOld => {
                    ApiError::SequenceNumberTooOld(Some(err.error.message))
                }
                AptosErrorCode::VmError => ApiError::VmError(Some(err.error.message)),
                AptosErrorCode::HealthCheckFailed => {
                    ApiError::InternalError(Some(err.error.message))
                }
                AptosErrorCode::MempoolIsFull => ApiError::MempoolIsFull(Some(err.error.message)),
                AptosErrorCode::WebFrameworkError => {
                    ApiError::InternalError(Some(err.error.message))
                }
                AptosErrorCode::BcsNotSupported => ApiError::InvalidInput(Some(err.error.message)),
                AptosErrorCode::InternalError => ApiError::InternalError(Some(err.error.message)),
                AptosErrorCode::ApiDisabled => ApiError::InternalError(Some(err.error.message)),
            },
            RestError::Bcs(_) => ApiError::DeserializationFailed(None),
            RestError::Json(_) => ApiError::DeserializationFailed(None),
            RestError::WebClient(err) => ApiError::InternalError(Some(err.to_string())),
            RestError::UrlParse(err) => ApiError::InternalError(Some(err.to_string())),
            RestError::Timeout(err) => ApiError::InternalError(Some(err.to_string())),
            RestError::Unknown(err) => ApiError::InternalError(Some(err.to_string())),
        }
    }
}

impl From<AccountAddressParseError> for ApiError {
    fn from(err: AccountAddressParseError) -> Self {
        ApiError::DeserializationFailed(Some(err.to_string()))
    }
}

impl From<FromHexError> for ApiError {
    fn from(err: FromHexError) -> Self {
        ApiError::DeserializationFailed(Some(err.to_string()))
    }
}

impl From<bcs::Error> for ApiError {
    fn from(err: bcs::Error) -> Self {
        ApiError::DeserializationFailed(Some(err.to_string()))
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::InternalError(Some(err.to_string()))
    }
}

impl From<std::num::ParseIntError> for ApiError {
    fn from(err: std::num::ParseIntError) -> Self {
        ApiError::DeserializationFailed(Some(err.to_string()))
    }
}

impl warp::reject::Reject for ApiError {}

impl Reply for ApiError {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::json(&self.into_error()).into_response()
    }
}
