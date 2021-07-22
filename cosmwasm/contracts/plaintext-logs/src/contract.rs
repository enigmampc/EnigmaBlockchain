use cosmwasm_std::{
    log, plaintext_log, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage,
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg};

pub fn init<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: HandleMsg,
) -> StdResult<HandleResponse> {
    let response = HandleResponse {
        messages: vec![],
        data: None,
        log: vec![
            log("encrypted log", "encrypted value"),
            log("ZW5jb2RlZCBsb2cK", "ZW5jb2RlZCB2YWx1ZQo="), // base64
            plaintext_log("plaintext log", "plaintext value"),
        ],
    };
    Ok(response)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _msg: QueryMsg,
) -> StdResult<Binary> {
    Err(StdError::generic_err(""))
}
