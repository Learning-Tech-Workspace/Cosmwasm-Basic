use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

use msg::{ExecMsg, InitMsg, QueryMsg};

mod contract;
pub mod msg;
mod state;
mod test;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    // like the main function
    // call when first time create
    use contract::instantiate;
    instantiate(deps, env, msg.counter, msg.minimal_donation);
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
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecMsg) -> StdResult<Response> {
    use contract::execute::{donate, reset, withdraw};
    use msg::ExecMsg::*;
    match msg {
        Donate {} => donate(deps, info),
        Reset { value } => reset(deps, info, value),
        Withdraw {} => withdraw(deps, env, info),
    }
}
