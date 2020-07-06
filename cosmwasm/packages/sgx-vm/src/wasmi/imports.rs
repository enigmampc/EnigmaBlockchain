//! This file should be autogenerated based on the headers created from the .edl file.

use enclave_ffi_types::{Ctx, EnclaveBuffer, HandleResult, InitResult, QueryResult};
use sgx_types::{sgx_enclave_id_t, sgx_status_t};

extern "C" {
    /// Copy a buffer into the enclave memory space, and receive an opaque pointer to it.
    pub fn ecall_allocate(
        eid: sgx_enclave_id_t,
        retval: *mut EnclaveBuffer,
        buffer: *const u8,
        length: usize,
    ) -> sgx_status_t;

    /// Trigger the init method in a wasm contract
    pub fn ecall_init(
        eid: sgx_enclave_id_t,
        retval: *mut InitResult,
        context: Ctx,
        gas_limit: u64,
        used_gas: *mut u64,
        contract: *const u8,
        contract_len: usize,
        env: *const u8,
        env_len: usize,
        msg: *const u8,
        msg_len: usize,
    ) -> sgx_status_t;

    /// Trigger a handle method in a wasm contract
    pub fn ecall_handle(
        eid: sgx_enclave_id_t,
        retval: *mut HandleResult,
        context: Ctx,
        gas_limit: u64,
        used_gas: *mut u64,
        contract: *const u8,
        contract_len: usize,
        env: *const u8,
        env_len: usize,
        msg: *const u8,
        msg_len: usize,
    ) -> sgx_status_t;

    /// Trigger a query method in a wasm contract
    pub fn ecall_query(
        eid: sgx_enclave_id_t,
        retval: *mut QueryResult,
        context: Ctx,
        gas_limit: u64,
        used_gas: *mut u64,
        contract: *const u8,
        contract_len: usize,
        msg: *const u8,
        msg_len: usize,
    ) -> sgx_status_t;
}
