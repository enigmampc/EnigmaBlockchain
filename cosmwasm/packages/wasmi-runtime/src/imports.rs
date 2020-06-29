//! This file should be autogenerated based on the headers created from the .edl file.

use enclave_ffi_types::{Ctx, EnclaveBuffer, OcallReturn, UntrustedVmError, UserSpaceBuffer};
use sgx_types::*;

extern "C" {
    pub fn ocall_allocate(
        retval: *mut UserSpaceBuffer,
        buffer: *const u8,
        length: usize,
    ) -> sgx_status_t;

    pub fn ocall_read_db(
        retval: *mut OcallReturn,
        context: Ctx,
        vm_error: *mut UntrustedVmError,
        gas_used: *mut u64,
        value: *mut EnclaveBuffer,
        key: *const u8,
        key_len: usize,
    ) -> sgx_status_t;

    pub fn ocall_remove_db(
        retval: *mut OcallReturn,
        context: Ctx,
        vm_error: *mut UntrustedVmError,
        gas_used: *mut u64,
        key: *const u8,
        key_len: usize,
    ) -> sgx_status_t;

    pub fn ocall_write_db(
        retval: *mut OcallReturn,
        context: Ctx,
        vm_error: *mut UntrustedVmError,
        gas_used: *mut u64,
        key: *const u8,
        key_len: usize,
        value: *const u8,
        value_len: usize,
    ) -> sgx_status_t;
}

extern "C" {
    pub fn ocall_sgx_init_quote(
        ret_val: *mut sgx_status_t,
        ret_ti: *mut sgx_target_info_t,
        ret_gid: *mut sgx_epid_group_id_t,
    ) -> sgx_status_t;
    pub fn ocall_get_ias_socket(ret_val: *mut sgx_status_t, ret_fd: *mut i32) -> sgx_status_t;
    pub fn ocall_get_quote(
        ret_val: *mut sgx_status_t,
        p_sigrl: *const u8,
        sigrl_len: u32,
        p_report: *const sgx_report_t,
        quote_type: sgx_quote_sign_type_t,
        p_spid: *const sgx_spid_t,
        p_nonce: *const sgx_quote_nonce_t,
        p_qe_report: *mut sgx_report_t,
        p_quote: *mut u8,
        maxlen: u32,
        p_quote_len: *mut u32,
    ) -> sgx_status_t;
}
