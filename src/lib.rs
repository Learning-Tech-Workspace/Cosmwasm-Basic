use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

use error::ContractError;
use msg::{ExecMsg, InitMsg, QueryMsg};

mod contract;
pub mod error;
pub mod msg;
mod state;
mod test;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    // like the main function
    // call when first time create
    use contract::instantiate;
    instantiate(deps, info, msg.counter, msg.minimal_donation);
    Ok(Response::new())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    use contract::query::{incremented, value};
    use msg::QueryMsg::*;
    match _msg {
        Value {} => to_binary(&value(deps)?),
        Incremented { value } => to_binary(&incremented(value)),
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecMsg,
) -> Result<Response, ContractError> {
    use contract::execute::{donate, reset, withdraw, withdraw_to};
    use msg::ExecMsg::*;
    match msg {
        Donate {} => donate(deps, info).map_err(ContractError::Std),
        Reset { value } => reset(deps, info, value),
        Withdraw {} => withdraw(deps, env, info),
        WithdrawTo {
            receiver,
            limit_funds,
        } => withdraw_to(deps, env, info, receiver, limit_funds),
    }
}
