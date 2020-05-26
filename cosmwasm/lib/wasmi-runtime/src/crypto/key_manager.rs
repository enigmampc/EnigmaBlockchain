use crate::consts::*;
use crate::crypto::keys::{KeyPair, Seed};
use crate::crypto::traits::*;
use enclave_ffi_types::{CryptoError, EnclaveError};
use lazy_static::lazy_static;
use log::*;

pub struct Keychain {
    consensus_seed: Option<Seed>,
    consensus_state_ikm: Option<Seed>,
    consensus_seed_exchange_keypair: Option<KeyPair>,
    consensus_io_exchange_keypair: Option<KeyPair>,
    registration_key: Option<KeyPair>,
}

lazy_static! {
    pub static ref KEY_MANAGER: Keychain = Keychain::new();
}

impl Keychain {
    pub fn new() -> Self {
        let consensus_seed = match Seed::unseal(CONSENSUS_SEED_SEALING_PATH) {
            Ok(k) => Some(k),
            Err(e) => None,
        };

        let registration_key = match KeyPair::unseal(REGISTRATION_KEY_SEALING_PATH) {
            Ok(k) => Some(k),
            Err(e) => None,
        };

        let mut x = Keychain {
            consensus_seed,
            registration_key,
            consensus_state_ikm: None,
            consensus_seed_exchange_keypair: None,
            consensus_io_exchange_keypair: None,
        };

        x.generate_consensus_master_keys();

        return x;

        // let kdf_salt: Vec<u8> = vec![
        //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x4b, 0xea, 0xd8, 0xdf,
        //     0x69, 0x99, 0x08, 0x52, 0xc2, 0x02, 0xdb, 0x0e, 0x00, 0x97, 0xc1, 0xa1, 0x2e, 0xa6,
        //     0x37, 0xd7, 0xe9, 0x6d,
        // ]; // Bitcoin halving block hash https://www.blockchain.com/btc/block/000000000000000000024bead8df69990852c202db0e0097c1a12ea637d7e96d
    }

    pub fn create_consensus_seed(&mut self) -> Result<(), CryptoError> {
        match Seed::new() {
            Ok(seed) => {
                if let Err(e) = self.set_consensus_seed(seed) {
                    return Err(CryptoError::KeyError);
                }
            }
            Err(err) => return Err(err),
        };
        Ok(())
    }

    pub fn create_registration_key(&mut self) -> Result<(), CryptoError> {
        match KeyPair::new() {
            Ok(key) => {
                if let Err(e) = self.set_registration_key(key) {
                    return Err(CryptoError::KeyError);
                }
            }
            Err(err) => return Err(err),
        };
        Ok(())
    }

    pub fn is_registration_key_set(&self) -> bool {
        return self.registration_key.is_some();
    }

    pub fn is_consensus_state_ikm_set(&self) -> bool {
        return self.consensus_state_ikm.is_some();
    }

    pub fn is_consensus_seed_exchange_keypair_set(&self) -> bool {
        return self.consensus_seed_exchange_keypair.is_some();
    }

    pub fn is_consensus_io_exchange_keypair_set(&self) -> bool {
        return self.consensus_io_exchange_keypair.is_some();
    }

    pub fn is_consensus_seed_set(&self) -> bool {
        return self.consensus_seed.is_some();
    }

    pub fn get_consensus_state_ikm(&self) -> Result<Seed, CryptoError> {
        if self.consensus_state_ikm.is_some() {
            Ok(self.consensus_state_ikm.unwrap())
        } else {
            error!("Error accessing base_state_key (does not exist, or was not initialized)");
            Err(CryptoError::ParsingError)
        }
    }

    pub fn get_consensus_seed(&self) -> Result<Seed, CryptoError> {
        if self.consensus_seed.is_some() {
            Ok(self.consensus_seed.unwrap())
        } else {
            error!("Error accessing consensus_seed (does not exist, or was not initialized)");
            Err(CryptoError::ParsingError)
        }
    }

    pub fn seed_exchange_key(&self) -> Result<KeyPair, CryptoError> {
        if self.consensus_seed_exchange_keypair.is_some() {
            // KeyPair does not implement copy (due to internal type not implementing it
            Ok(self.consensus_seed_exchange_keypair.clone().unwrap())
        } else {
            error!("Error accessing consensus_seed_exchange_keypair (does not exist, or was not initialized)");
            Err(CryptoError::ParsingError)
        }
    }

    pub fn get_consensus_io_exchange_keypair(&self) -> Result<KeyPair, CryptoError> {
        if self.consensus_io_exchange_keypair.is_some() {
            // KeyPair does not implement copy (due to internal type not implementing it
            Ok(self.consensus_io_exchange_keypair.clone().unwrap())
        } else {
            error!("Error accessing consensus_io_exchange_keypair (does not exist, or was not initialized)");
            Err(CryptoError::ParsingError)
        }
    }

    pub fn get_registration_key(&self) -> Result<KeyPair, CryptoError> {
        if self.registration_key.is_some() {
            // KeyPair does not implement copy (due to internal type not implementing it
            Ok(self.registration_key.clone().unwrap())
        } else {
            error!("Error accessing registration_key (does not exist, or was not initialized)");
            Err(CryptoError::ParsingError)
        }
    }

    pub fn set_registration_key(&mut self, kp: KeyPair) -> Result<(), EnclaveError> {
        if let Err(e) = kp.seal(REGISTRATION_KEY_SEALING_PATH) {
            error!("Error sealing registration key");
            return Err(e);
        }
        Ok(self.registration_key = Some(kp.clone()))
    }

    pub fn set_consensus_seed_exchange_keypair(&mut self, kp: KeyPair) {
        self.consensus_seed_exchange_keypair = Some(kp.clone())
    }

    pub fn set_consensus_io_exchange_keypair(&mut self, kp: KeyPair) {
        self.consensus_io_exchange_keypair = Some(kp.clone())
    }

    pub fn set_consensus_state_ikm(&mut self, consensus_state_ikm: Seed) {
        self.consensus_state_ikm = Some(consensus_state_ikm.clone());
    }

    pub fn set_consensus_seed(&mut self, consensus_seed: Seed) -> Result<(), EnclaveError> {
        if let Err(e) = consensus_seed.seal(CONSENSUS_SEED_SEALING_PATH) {
            error!("Error sealing consensus_seed");
            return Err(e);
        }
        Ok(self.consensus_seed = Some(consensus_seed.clone()))
    }

    pub fn generate_consensus_master_keys(&mut self) -> Result<(), EnclaveError> {
        if !self.is_consensus_seed_set() {
            debug!("Seed not initialized! Cannot derive enclave keys");
            return Ok(());
        }

        // consensus_seed_exchange_keypair

        let consensus_seed_exchange_keypair_bytes = self
            .consensus_seed
            .unwrap()
            .derive_key_from_this(CONSENSUS_SEED_EXCHANGE_KEYPAIR_DERIVE_ORDER);
        let consensus_seed_exchange_keypair =
            KeyPair::new_from_slice(&consensus_seed_exchange_keypair_bytes).map_err(|err| {
                error!(
                    "[Enclave] Error creating consensus_seed_exchange_keypair: {:?}",
                    err
                );
                EnclaveError::FailedUnseal /* change error type? */
            })?;

        self.set_consensus_seed_exchange_keypair(consensus_seed_exchange_keypair);

        // consensus_io_exchange_keypair

        let consensus_io_exchange_keypair_bytes = self
            .consensus_seed
            .unwrap()
            .derive_key_from_this(CONSENSUS_IO_EXCHANGE_KEYPAIR_DERIVE_ORDER);
        let consensus_io_exchange_keypair =
            KeyPair::new_from_slice(&consensus_io_exchange_keypair_bytes).map_err(|err| {
                error!(
                    "[Enclave] Error creating consensus_io_exchange_keypair: {:?}",
                    err
                );
                EnclaveError::FailedUnseal /* change error type? */
            })?;

        self.set_consensus_io_exchange_keypair(consensus_io_exchange_keypair);

        // consensus_state_ikm

        let consensus_state_ikm_bytes = self
            .consensus_seed
            .unwrap()
            .derive_key_from_this(CONSENSUS_STATE_IKM_DERIVE_ORDER);
        let consensus_state_ikm = Seed::new_from_slice(&consensus_state_ikm_bytes);

        self.set_consensus_state_ikm(consensus_state_ikm);

        Ok(())
    }
}
