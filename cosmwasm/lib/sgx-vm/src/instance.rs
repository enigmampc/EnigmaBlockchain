use std::env;
use std::marker::PhantomData;
use std::path::Path;
use std::str;

use log::*;
use sgx_types::{
    sgx_attributes_t, sgx_launch_token_t, sgx_misc_attribute_t, sgx_status_t, SgxResult,
};
use sgx_urts::SgxEnclave;

use cosmwasm::traits::Api;
use lazy_static::lazy_static;

use crate::attestation::{inner_create_report, inner_get_encrypted_seed};
use crate::errors::{Error, Result};
use crate::seed::{inner_init_bootstrap, inner_init_seed, inner_key_gen};
use crate::wasmi::Module;
use crate::ENCRYPTED_SEED_SIZE;
use crate::{Extern, Storage};

/// An instance is a combination of wasm code, storage, and gas limit.
pub struct Instance<S: Storage + 'static, A: Api + 'static> {
    enclave_instance: Module,
    pub api: A,
    // This does not store data but only fixes type information
    type_storage: PhantomData<S>,
}

static ENCLAVE_FILE: &'static str = "librust_cosmwasm_enclave.signed.so";

// this is here basically to be able to call the enclave initialization -- we can move this somewhere else and simplify

pub fn init_seed_u(
    master_cert: *const u8,
    master_cert_len: u32,
    encrypted_seed: *const u8,
    encrypted_seed_len: u32,
) -> SgxResult<sgx_status_t> {
    info!("Hello from just before initializing - produce_report");
    let enclave = init_enclave().unwrap();
    info!("Hello from just after initializing - produce_report");

    inner_init_seed(
        enclave.geteid(),
        master_cert,
        master_cert_len,
        encrypted_seed,
        encrypted_seed_len,
    )
}

pub fn create_attestation_report_u() -> SgxResult<sgx_status_t> {
    info!("Hello from just before initializing - create_attestation_report_u");
    let enclave = init_enclave().unwrap();
    info!("Hello from just after initializing - create_attestation_report_u");

    inner_create_report(enclave.geteid())
}

pub fn untrusted_key_gen() -> SgxResult<[u8; 64]> {
    info!("Hello from just before initializing - untrusted_init_bootstrap");
    let enclave = init_enclave().unwrap();
    info!("Hello from just after initializing - untrusted_init_bootstrap");

    inner_key_gen(enclave.geteid())
}

pub fn untrusted_init_bootstrap() -> SgxResult<[u8; 64]> {
    info!("Hello from just before initializing - untrusted_init_bootstrap");
    let enclave = init_enclave().unwrap();
    info!("Hello from just after initializing - untrusted_init_bootstrap");

    inner_init_bootstrap(enclave.geteid())
}

pub fn untrusted_get_encrypted_seed(cert: &[u8]) -> SgxResult<[u8; ENCRYPTED_SEED_SIZE]> {
    info!("Hello from just before initializing - untrusted_get_encrypted_seed");
    let enclave = init_enclave().unwrap();
    info!("Hello from just after initializing - untrusted_get_encrypted_seed");

    inner_get_encrypted_seed(enclave.geteid(), cert.as_ptr(), cert.len() as u32)
}

pub fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };

    // Step : try to create a .enigma folder for storing all the files
    // Create a directory, returns `io::Result<()>`
    let enclave_directory = env::var("SCRT_ENCLAVE_DIR").unwrap_or('.'.to_string());

    let path = Path::new(&enclave_directory);

    let enclave_file_path: std::path::PathBuf = path.join(ENCLAVE_FILE);

    SgxEnclave::create(
        enclave_file_path,
        debug,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr,
    )
}

lazy_static! {
    pub static ref SGX_ENCLAVE: SgxResult<SgxEnclave> = init_enclave();
}

impl<S, A> Instance<S, A>
where
    S: Storage + 'static,
    A: Api + 'static,
{
    pub fn from_code(code: &[u8], deps: Extern<S, A>, gas_limit: u64) -> Result<Self> {
        let enclave = SGX_ENCLAVE
            .as_ref()
            .map_err(|err| Error::SdkErr { inner: *err })?;

        let module = Module::new(code.to_vec(), gas_limit, enclave);

        Ok(Instance::from_wasmer(module, deps, gas_limit))
    }

    /*
    pub fn from_module(module: &Module, deps: Extern<S, A>, gas_limit: u64) -> Result<Self> {
        // copy this so it can be moved into the closures, without pulling in deps
        let api = deps.api;
        let import_obj = imports! {
            || { setup_context::<S>() },
            "env" => {
                // Reads the database entry at the given key into the the value.
                // A prepared and sufficiently large memory Region is expected at value_ptr that points to pre-allocated memory.
                // Returns length of the value in bytes on success. Returns negative value on error. An incomplete list of error codes is:
                //   value region too small: -1000002
                // Ownership of both input and output pointer is not transferred to the host.
                "read_db" => Func::new(move |ctx: &mut Ctx, key_ptr: u32, value_ptr: u32| -> i32 {
                    do_read::<S>(ctx, key_ptr, value_ptr)
                }),
                // Writes the given value into the database entry at the given key.
                // Ownership of both input and output pointer is not transferred to the host.
                "write_db" => Func::new(move |ctx: &mut Ctx, key_ptr: u32, value_ptr: u32| {
                    do_write::<S>(ctx, key_ptr, value_ptr)
                }),
                // Reads human address from human_ptr and writes canonicalized representation to canonical_ptr.
                // A prepared and sufficiently large memory Region is expected at canonical_ptr that points to pre-allocated memory.
                // Returns negative value on error. Returns length of the canoncal address on success.
                // Ownership of both input and output pointer is not transferred to the host.
                "canonicalize_address" => Func::new(move |ctx: &mut Ctx, human_ptr: u32, canonical_ptr: u32| -> i32 {
                    do_canonical_address(api, ctx, human_ptr, canonical_ptr)
                }),
                // Reads canonical address from canonical_ptr and writes humanized representation to human_ptr.
                // A prepared and sufficiently large memory Region is expected at human_ptr that points to pre-allocated memory.
                // Returns negative value on error. Returns length of the human address on success.
                // Ownership of both input and output pointer is not transferred to the host.
                "humanize_address" => Func::new(move |ctx: &mut Ctx, canonical_ptr: u32, human_ptr: u32| -> i32 {
                    do_human_address(api, ctx, canonical_ptr, human_ptr)
                }),
            },
        };
        let wasmer_instance = module.instantiate(&import_obj).context(WasmerErr {})?;
        Ok(Instance::from_wasmer(wasmer_instance, deps, gas_limit))
    }
    */

    pub fn from_wasmer(
        mut module: Module,
        deps: Extern<S, A>,
        _gas_limit: u64, // gas_limit parameter left here for code compatibility with original library.
    ) -> Self {
        module.set_storage(Box::new(deps.storage) as Box<dyn Storage>);
        Instance {
            enclave_instance: module,
            api: deps.api,
            type_storage: PhantomData::<S> {},
        }
    }

    /// Takes ownership of instance and decomposes it into its components.
    /// The components we want to preserve are returned, the rest is dropped.
    pub fn recycle(mut instance: Self) -> (Module, Option<Extern<S, A>>) {
        let ext = if let Some(storage) = instance.enclave_instance.take_storage() {
            let storage = *storage
                .downcast::<S>()
                .ok()
                .expect("We only ever provide storage of type S");
            Some(Extern {
                storage,
                api: instance.api,
            })
        } else {
            None
        };
        (instance.enclave_instance, ext)
    }

    pub fn get_gas(&self) -> u64 {
        self.enclave_instance.gas_limit()
    }

    pub fn with_storage<F: FnMut(&mut S)>(&mut self, mut func: F) {
        func(
            self.enclave_instance
                .storage_mut()
                .downcast_mut::<S>()
                .expect("We only ever provide storage of type S"),
        );
    }

    pub fn call_init(&mut self, env: &[u8], msg: &[u8]) -> Result<Vec<u8>, Error> {
        let init_result = self.enclave_instance.init(env, msg)?;
        // TODO verify signature
        Ok(init_result.into_output())
    }

    pub fn call_handle(&mut self, env: &[u8], msg: &[u8]) -> Result<Vec<u8>, Error> {
        let init_result = self.enclave_instance.handle(env, msg)?;
        // TODO verify signature
        Ok(init_result.into_output())
    }

    pub fn call_query(&mut self, msg: &[u8]) -> Result<Vec<u8>, Error> {
        let init_result = self.enclave_instance.query(msg)?;
        // TODO verify signature
        Ok(init_result.into_output())
    }

    /*
    pub fn memory(&self, ptr: u32) -> Vec<u8> {
        read_region(self.wasmer_instance.context(), ptr)
    }

    // allocate memory in the instance and copies the given data in
    // returns the memory offset, to be later passed as an argument
    pub fn allocate(&mut self, data: &[u8]) -> Result<u32> {
        let alloc: Func<u32, u32> = self.func("allocate")?;
        let ptr = alloc.call(data.len() as u32).context(RuntimeErr {})?;
        write_region(self.wasmer_instance.context(), ptr, data)?;
        Ok(ptr)
    }

    // deallocate frees memory in the instance and that was either previously
    // allocated by us, or a pointer from a return value after we copy it into rust.
    // we need to clean up the wasm-side buffers to avoid memory leaks
    pub fn deallocate(&mut self, ptr: u32) -> Result<()> {
        let dealloc: Func<u32, ()> = self.func("deallocate")?;
        dealloc.call(ptr).context(RuntimeErr {})?;
        Ok(())
    }

    pub fn func<Args, Rets>(&self, name: &str) -> Result<Func<Args, Rets, Wasm>>
    where
        Args: WasmTypeList,
        Rets: WasmTypeList,
    {
        self.wasmer_instance.func(name).context(ResolveErr {})
    }
    */
}

#[cfg(test)]
mod test {
    use cosmwasm::mock::mock_env;
    use cosmwasm::types::coin;

    use crate::calls::{call_handle, call_init, call_query};
    use crate::testing::{mock_instance, mock_instance_with_gas_limit};

    static CONTRACT_0_7: &[u8] = include_bytes!("../testdata/contract_0.7.wasm");

    #[test]
    #[cfg(feature = "default-cranelift")]
    fn set_get_and_gas_cranelift_noop() {
        let instance = mock_instance_with_gas_limit(&CONTRACT_0_7, 123321);
        let orig_gas = instance.get_gas();
        assert_eq!(orig_gas, 1_000_000);
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn set_get_and_gas_singlepass_works() {
        let instance = mock_instance_with_gas_limit(&CONTRACT_0_7, 123321);
        let orig_gas = instance.get_gas();
        assert_eq!(orig_gas, 123321);
    }

    #[test]
    #[should_panic]
    fn with_context_safe_for_panic() {
        // this should fail with the assertion, but not cause a double-free crash (issue #59)
        let instance = mock_instance(&CONTRACT_0_7);
        instance.with_storage(|_store| assert_eq!(1, 2));
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn contract_deducts_gas_init() {
        let mut instance = mock_instance(&CONTRACT_0_7);
        let orig_gas = instance.get_gas();

        // init contract
        let env = mock_env(&instance.api, "creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init(&mut instance, &env, msg).unwrap();

        let init_used = orig_gas - instance.get_gas();
        println!("init used: {}", init_used);
        assert_eq!(init_used, 52_543);
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn contract_deducts_gas_handle() {
        let mut instance = mock_instance(&CONTRACT_0_7);

        // init contract
        let env = mock_env(&instance.api, "creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init(&mut instance, &env, msg).unwrap();

        // run contract - just sanity check - results validate in contract unit tests
        let gas_before_handle = instance.get_gas();
        let env = mock_env(
            &instance.api,
            "verifies",
            &coin("15", "earth"),
            &coin("1015", "earth"),
        );
        let msg = br#"{"release":{}}"#;
        call_handle(&mut instance, &env, msg).unwrap();

        let handle_used = gas_before_handle - instance.get_gas();
        println!("handle used: {}", handle_used);
        assert_eq!(handle_used, 91_487);
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn contract_enforces_gas_limit() {
        let mut instance = mock_instance_with_gas_limit(&CONTRACT_0_7, 20_000);

        // init contract
        let env = mock_env(&instance.api, "creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let res = call_init(&mut instance, &env, msg);
        assert!(res.is_err());
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn query_works_with_metering() {
        let mut instance = mock_instance(&CONTRACT_0_7);

        // init contract
        let env = mock_env(&instance.api, "creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let _res = call_init(&mut instance, &env, msg).unwrap().unwrap();

        // run contract - just sanity check - results validate in contract unit tests
        let gas_before_query = instance.get_gas();
        // we need to encode the key in base64
        let msg = r#"{"verifier":{}}"#.as_bytes();
        let res = call_query(&mut instance, msg).unwrap();
        let answer = res.unwrap();
        assert_eq!(answer.as_slice(), "verifies".as_bytes());

        let query_used = gas_before_query - instance.get_gas();
        println!("query used: {}", query_used);
        assert_eq!(query_used, 44_921);
    }
}
