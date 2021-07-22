//! must keep this file in sync with cosmwasm/packages/std/src/types.rs and cosmwasm/packages/std/src/init_handle.rs

#![allow(unused)]

/// These types are are copied over from the cosmwasm_std package, and must be kept in sync with it.
///
/// We copy these types instead of directly depending on them, because we require special versions of serde
/// inside the enclave, which are different from the versions that cosmwasm_std uses.
/// For some reason patching the dependencies didn't work, so we are forced to maintain this copy, for now :(
use std::fmt;

use serde::{Deserialize, Serialize};

use super::encoding::Binary;
use crate::consts::BECH32_PREFIX_ACC_ADDR;
use crate::crypto::multisig::MultisigThresholdPubKey;
use crate::crypto::secp256k1::Secp256k1PubKey;
use crate::crypto::traits::PubKey;
use crate::crypto::CryptoError;
use bech32::{FromBase32, ToBase32};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct HumanAddr(pub String);

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct CanonicalAddr(pub Binary);

impl HumanAddr {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn from_canonical(canonical_addr: &CanonicalAddr) -> Result<Self, bech32::Error> {
        let human_addr_str = bech32::encode(
            BECH32_PREFIX_ACC_ADDR,
            canonical_addr.as_slice().to_base32(),
        )?;

        Ok(HumanAddr(human_addr_str))
    }
}

impl fmt::Display for HumanAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl From<&str> for HumanAddr {
    fn from(addr: &str) -> Self {
        HumanAddr(addr.to_string())
    }
}

impl From<&HumanAddr> for HumanAddr {
    fn from(addr: &HumanAddr) -> Self {
        HumanAddr(addr.0.to_string())
    }
}

impl CanonicalAddr {
    pub fn as_slice(&self) -> &[u8] {
        &self.0.as_slice()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn from_human(human_addr: &HumanAddr) -> Result<Self, bech32::Error> {
        let (decoded_prefix, data) = bech32::decode(human_addr.as_str())?;
        let canonical = Vec::<u8>::from_base32(&data)?;

        Ok(CanonicalAddr(Binary(canonical)))
    }
}

impl fmt::Display for CanonicalAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Env {
    pub block: BlockInfo,
    pub message: MessageInfo,
    pub contract: ContractInfo,
    pub contract_key: Option<String>,
    #[serde(default)]
    pub contract_code_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct BlockInfo {
    pub height: u64,
    // time is seconds since epoch begin (Jan. 1, 1970)
    pub time: u64,
    pub chain_id: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct MessageInfo {
    /// The `sender` field from the wasm/MsgStoreCode, wasm/MsgInstantiateContract or wasm/MsgExecuteContract message.
    /// You can think of this as the address that initiated the action (i.e. the message). What that
    /// means exactly heavily depends on the application.
    ///
    /// The x/wasm module ensures that the sender address signed the transaction.
    /// Additional signers of the transaction that are either needed for other messages or contain unnecessary
    /// signatures are not propagated into the contract.
    ///
    /// There is a discussion to open up this field to multiple initiators, which you're welcome to join
    /// if you have a specific need for that feature: https://github.com/CosmWasm/cosmwasm/issues/293
    pub sender: HumanAddr,
    pub sent_funds: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct ContractInfo {
    pub address: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct Coin {
    pub denom: String,
    pub amount: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum WasmOutput {
    ErrObject {
        #[serde(rename = "Err")]
        err: Value,
    },
    OkString {
        #[serde(rename = "Ok")]
        ok: String,
    },
    OkObject {
        #[serde(rename = "Ok")]
        ok: ContractResult,
    },
}

// This should be in correlation with cosmwasm-std/init_handle's InitResponse and HandleResponse
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ContractResult {
    pub messages: Vec<CosmosMsg>,
    pub log: Vec<LogAttribute>,
    pub data: Option<Binary>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
// This should be in correlation with cosmwasm-std/init_handle's CosmosMsg
// See https://github.com/serde-rs/serde/issues/1296 why we cannot add De-Serialize trait bounds to T
pub enum CosmosMsg<T = CustomMsg>
where
    T: Clone + fmt::Debug + PartialEq,
{
    Bank(BankMsg),
    // by default we use RawMsg, but a contract can override that
    // to call into more app-specific code (whatever they define)
    Custom(T),
    Staking(StakingMsg),
    Wasm(WasmMsg),
    Gov(GovMsg),
}

/// Added this here for reflect tests....
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
/// CustomMsg is an override of CosmosMsg::Custom to show this works and can be extended in the contract
pub enum CustomMsg {
    Debug(String),
    Raw(Binary),
}

impl Into<CosmosMsg<CustomMsg>> for CustomMsg {
    fn into(self) -> CosmosMsg<CustomMsg> {
        CosmosMsg::Custom(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GovMsg {
    // Let contract vote on a governance proposal
    Vote {
        proposal: u64,
        vote_option: VoteOption,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum VoteOption {
    Yes,
    No,
    Abstain,
    NoWithVeto,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BankMsg {
    // this moves tokens in the underlying sdk
    Send {
        from_address: HumanAddr,
        to_address: HumanAddr,
        amount: Vec<Coin>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StakingMsg {
    Delegate {
        // delegator is automatically set to address of the calling contract
        validator: HumanAddr,
        amount: Coin,
    },
    Undelegate {
        // delegator is automatically set to address of the calling contract
        validator: HumanAddr,
        amount: Coin,
    },
    Withdraw {
        // delegator is automatically set to address of the calling contract
        validator: HumanAddr,
        /// this is the "withdraw address", the one that should receive the rewards
        /// if None, then use delegator address
        recipient: Option<HumanAddr>,
    },
    Redelegate {
        // delegator is automatically set to address of the calling contract
        src_validator: HumanAddr,
        dst_validator: HumanAddr,
        amount: Coin,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WasmMsg {
    /// this dispatches a call to another contract at a known address (with known ABI)
    Execute {
        contract_addr: HumanAddr,
        /// callback_code_hash is the hex encoded hash of the code. This is used by Secret Network to harden against replaying the contract
        /// It is used to bind the request to a destination contract in a stronger way than just the contract address which can be faked
        callback_code_hash: String,
        /// msg is the json-encoded HandleMsg struct (as raw Binary)
        msg: Binary,
        send: Vec<Coin>,
        callback_sig: Option<Vec<u8>>,
    },
    /// this instantiates a new contracts from previously uploaded wasm code
    Instantiate {
        code_id: u64,
        /// callback_code_hash is the hex encoded hash of the code. This is used by Secret Network to harden against replaying the contract
        /// It is used to bind the request to a destination contract in a stronger way than just the contract address which can be faked
        callback_code_hash: String,
        /// msg is the json-encoded InitMsg struct (as raw Binary)
        msg: Binary,
        send: Vec<Coin>,
        /// Human-readable label for the contract
        #[serde(default)]
        label: String,
        callback_sig: Option<Vec<u8>>,
    },
}

impl<T: Clone + fmt::Debug + PartialEq> From<GovMsg> for CosmosMsg<T> {
    fn from(msg: GovMsg) -> Self {
        CosmosMsg::Gov(msg)
    }
}

impl<T: Clone + fmt::Debug + PartialEq> From<BankMsg> for CosmosMsg<T> {
    fn from(msg: BankMsg) -> Self {
        CosmosMsg::Bank(msg)
    }
}

#[cfg(feature = "staking")]
impl<T: Clone + fmt::Debug + PartialEq> From<StakingMsg> for CosmosMsg<T> {
    fn from(msg: StakingMsg) -> Self {
        CosmosMsg::Staking(msg)
    }
}

impl<T: Clone + fmt::Debug + PartialEq> From<WasmMsg> for CosmosMsg<T> {
    fn from(msg: WasmMsg) -> Self {
        CosmosMsg::Wasm(msg)
    }
}

/// Return true
///
/// Only used for serde annotations
fn bool_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct LogAttribute {
    pub key: String,
    pub value: String,
    /// nonstandard late addition, thus optional and only used in deserialization.
    /// The contracts may return this in newer versions that support distinguishing
    /// encrypted and plaintext logs. We naturally default to encrypted logs, and
    /// don't serialize the field later so it doesn't leak up to the Go layers.
    #[serde(default = "bool_true")]
    #[serde(skip_serializing)]
    pub encrypted: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QueryResult {
    Ok(Binary),
    Err(String),
}

impl QueryResult {
    // unwrap will panic on err, or give us the real data useful for tests
    pub fn unwrap(self) -> Binary {
        match self {
            QueryResult::Err(msg) => panic!("Unexpected error: {}", msg),
            QueryResult::Ok(res) => res,
        }
    }

    pub fn is_err(&self) -> bool {
        matches!(self, QueryResult::Err(_))
    }
}

// coin is a shortcut constructor for a set of one denomination of coins
pub fn coin(amount: &str, denom: &str) -> Vec<Coin> {
    vec![Coin {
        amount: amount.to_string(),
        denom: denom.to_string(),
    }]
}

/// A shorthand to produce a log attribute
pub fn log<K: ToString, V: ToString>(key: K, value: V) -> LogAttribute {
    LogAttribute {
        key: key.to_string(),
        value: value.to_string(),
        encrypted: true,
    }
}

/// A shorthand to produce a plaintext log attribute
pub fn plaintext_log<K: ToString, V: ToString>(key: K, value: V) -> LogAttribute {
    LogAttribute {
        key: key.to_string(),
        value: value.to_string(),
        encrypted: false,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CosmosSignature {
    // pub_key is an enum, because it can't be a boxed trait object (or something similar)
    // because it has to be Sized
    pub_key: PubKeyKind,
    signature: Binary,
}

impl CosmosSignature {
    pub fn get_public_key(&self) -> PubKeyKind {
        self.pub_key.clone()
    }

    pub fn get_signature(&self) -> Binary {
        self.signature.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum PubKeyKind {
    Secp256k1(Secp256k1PubKey),
    Multisig(MultisigThresholdPubKey),
}

impl PubKey for PubKeyKind {
    fn get_address(&self) -> CanonicalAddr {
        match self {
            PubKeyKind::Secp256k1(pubkey) => pubkey.get_address(),
            PubKeyKind::Multisig(pubkey) => pubkey.get_address(),
        }
    }

    fn bytes(&self) -> Vec<u8> {
        match self {
            PubKeyKind::Secp256k1(pubkey) => pubkey.bytes(),
            PubKeyKind::Multisig(pubkey) => pubkey.bytes(),
        }
    }

    fn verify_bytes(&self, bytes: &[u8], sig: &[u8]) -> Result<(), CryptoError> {
        match self {
            PubKeyKind::Secp256k1(pubkey) => pubkey.verify_bytes(bytes, sig),
            PubKeyKind::Multisig(pubkey) => pubkey.verify_bytes(bytes, sig),
        }
    }
}

// Should be in sync with https://github.com/cosmos/cosmos-sdk/blob/v0.38.3/x/auth/types/stdtx.go#L216
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct SignDoc {
    pub account_number: String,
    pub chain_id: String,
    pub fee: Value,
    pub memo: String,
    pub msgs: Vec<SignDocWasmMsg>,
    pub sequence: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SigInfo {
    pub sign_bytes: Binary,
    pub signature: CosmosSignature,
    pub callback_sig: Option<Binary>,
}

// This struct is basically the smae as WasmMsg, but serializes/deserializes differently
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum SignDocWasmMsg {
    #[serde(alias = "wasm/MsgExecuteContract")]
    Execute {
        contract: HumanAddr,
        /// msg is the json-encoded HandleMsg struct (as raw Binary)
        msg: String,
        sent_funds: Vec<Coin>,
        callback_sig: Option<Vec<u8>>,
    },
    #[serde(alias = "wasm/MsgInstantiateContract")]
    Instantiate {
        code_id: String,
        init_msg: String,
        init_funds: Vec<Coin>,
        label: Option<String>,
        callback_sig: Option<Vec<u8>>,
    },
}
