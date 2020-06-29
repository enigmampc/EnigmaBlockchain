#![allow(unused)]

use derive_more::Display;
use enclave_ffi_types::EnclaveError;
use wasmi::{Error as InterpreterError, HostError};

#[derive(Debug, Display)]
#[non_exhaustive]
pub enum WasmEngineError {
    FailedOcall,
    OutOfGas,
    EncryptionError,
    DecryptionError,
    DbError(DbError),
    MemoryAllocationError,
    MemoryReadError,
    MemoryWriteError,
    InputInvalid,
    InputEmpty,
    InputWrongPrefix,
    InputWrongLength,
    OutputWrongLength,
    NonExistentImportFunction,
    NotImplemented,
}

#[derive(Debug, Display)]
#[non_exhaustive]
pub enum DbError {
    FailedRead,
    FailedRemove,
    FailedWrite,
    FailedEncryption,
    FailedDecryption,
}

impl From<DbError> for WasmEngineError {
    fn from(err: DbError) -> Self {
        WasmEngineError::DbError(err)
    }
}

impl HostError for WasmEngineError {}

pub fn wasmi_error_to_enclave_error(wasmi_error: InterpreterError) -> EnclaveError {
    match wasmi_error
        .as_host_error()
        .map(|err| err.downcast_ref::<WasmEngineError>())
    {
        // An ocall failed during contract execution.
        Some(Some(WasmEngineError::FailedOcall)) => EnclaveError::FailedOcall,
        // Ran out of gas
        Some(Some(WasmEngineError::OutOfGas)) => EnclaveError::OutOfGas,
        Some(Some(WasmEngineError::EncryptionError)) => EnclaveError::FailedSeal,
        Some(Some(WasmEngineError::DecryptionError)) => EnclaveError::FailedUnseal,
        Some(Some(WasmEngineError::DbError(_))) => EnclaveError::FailedFunctionCall,
        Some(Some(WasmEngineError::NotImplemented)) => EnclaveError::NotImplemented,
        // Unexpected WasmEngineError variant or unexpected HostError.
        Some(None) => EnclaveError::Unknown,
        // The error is not a HostError. In the future we might want to return more specific errors.
        None => EnclaveError::FailedFunctionCall,
        _ => EnclaveError::Unknown,
    }
}