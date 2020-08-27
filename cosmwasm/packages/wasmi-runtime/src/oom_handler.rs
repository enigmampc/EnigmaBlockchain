use core::sync::atomic::{AtomicBool, Ordering};
use enclave_ffi_types::EnclaveError;
use lazy_static::lazy_static;
use std::backtrace::{self, PrintFormat};
use std::sync::SgxMutex;
/// SafetyBuffer is meant to occupy space on the heap, so when a memory
/// allocation fails we will free this buffer to allow safe panic unwinding
/// This is needed because while unwinding from panic some destructors try
/// to allocate more memory which causes a double fault. This way we can
/// make sure the unwind process has enough free memory to work properly.
struct SafetyBuffer {
    chunks: usize,
    min_chunks: usize,
    buffer: Vec<Vec<u8>>,
}

impl SafetyBuffer {
    /// Allocate `chunks` KiB on the heap
    pub fn new(chunks: usize, min_chunks: usize) -> Self {
        SafetyBuffer {
            chunks,
            min_chunks,
            buffer: SafetyBuffer::build_buffer(chunks, min_chunks).unwrap(),
        }
    }

    /// Free the buffer to allow panic to safely unwind
    pub fn clear(&mut self) {
        let mut temp = Vec::new();
        std::mem::swap(&mut self.buffer, &mut temp);
        drop(temp)
    }

    fn build_buffer(chunks: usize, min_chunks: usize) -> Result<Vec<Vec<u8>>, EnclaveError> {
        let mut buffer: Vec<Vec<u8>> = Vec::with_capacity(chunks);
        SafetyBuffer::top_up_buffer(&mut buffer, chunks, min_chunks)?;
        Ok(buffer)
    }

    fn top_up_buffer(
        buffer: &mut Vec<Vec<u8>>,
        chunks: usize,
        min_chunks: usize,
    ) -> Result<(), EnclaveError> {
        for i in buffer.len()..chunks {
            let mut kb: Vec<u8> = Vec::new();
            match kb.try_reserve_exact(1024) {
                Ok(_) => { /* continue */ }
                Err(_err) => {
                    if i > min_chunks {
                        break;
                    } else {
                        return Err(EnclaveError::MemorySafetyAllocationError);
                    }
                }
            };
            buffer.push(kb)
        }
        Ok(())
    }

    // Reallocate the buffer, use this after a successful unwind
    pub fn restore(&mut self) -> Result<(), EnclaveError> {
        if self.buffer.capacity() < self.chunks {
            SafetyBuffer::top_up_buffer(&mut self.buffer, self.chunks, self.min_chunks)?;
        }
        Ok(())
    }
}

lazy_static! {
    /// SAFETY_BUFFER is a 32 MiB of SafetyBuffer. This is occupying 50% of available memory
    /// to be extra sure this is enough.
    /// 2 MiB is the minimum allowed buffer. If we don't succeed to allocate 2 MiB, we throw a panic,
    /// if we do succeed to allocate 2 MiB but less than 32 MiB than we move on and will try to allocate
    /// the rest on the next entry to the enclave.
    static ref SAFETY_BUFFER: SgxMutex<SafetyBuffer> = SgxMutex::new(SafetyBuffer::new(16 * 1024, 2 * 1024));
}

static OOM_HAPPENED: AtomicBool = AtomicBool::new(false);

pub fn register_oom_handler() -> Result<(), EnclaveError> {
    let _ = backtrace::enable_backtrace("librust_cosmwasm_enclave.signed.so", PrintFormat::Full);

    {
        SAFETY_BUFFER.lock().unwrap().restore()?;
    }

    get_then_clear_oom_happened();

    std::alloc::set_alloc_error_hook(|layout| {
        OOM_HAPPENED.store(true, Ordering::SeqCst);

        {
            SAFETY_BUFFER.lock().unwrap().clear();
        }

        panic!(
            "SGX: Memory allocation of {} bytes failed. Trying to recover...\n",
            layout.size()
        );
    });

    Ok(())
}

pub fn get_then_clear_oom_happened() -> bool {
    OOM_HAPPENED.swap(false, Ordering::SeqCst)
}

pub fn restore_safety_buffer() -> Result<(), EnclaveError> {
    SAFETY_BUFFER.lock().unwrap().restore()
}
