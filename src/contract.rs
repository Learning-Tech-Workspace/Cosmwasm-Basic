use cosmwasm_std::{DepsMut, Response, StdResult};

use crate::state::COUNTER;

pub fn instantiate(deps: DepsMut, counter: u64) -> StdResult<Response> {
    COUNTER.save(deps.storage, &counter);
    Ok(Response::new())
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::{msg::ValueResp, state::COUNTER};

    pub fn value(deps: Deps) -> StdResult<ValueResp> {
        let value = COUNTER.load(deps.storage)?;
        Ok(ValueResp { value })
    }

    pub fn incremented(value: u64) -> ValueResp {
        ValueResp { value: value + 1 }
    }
}

pub mod execute {
    use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult};

    use crate::state::COUNTER;

    pub fn poke(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let mut counter = COUNTER.load(deps.storage)? + 1;
        COUNTER.save(deps.storage, &counter);

        // COUNTER.update(deps.storage, |counter| -> StdResult<_> { Ok(counter + 1) })?;

        let resp = Response::new()
            .add_attribute("action", "poke")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", counter.to_string());

        Ok(resp)
    }

    pub fn reset(deps: DepsMut, info: MessageInfo, value: u64) -> StdResult<Response> {
        COUNTER.update(deps.storage, |counter| -> StdResult<_> { Ok(value) })?;

        let resp = Response::new()
            .add_attribute("action", "reset")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", value.to_string());
        Ok(resp)
    }
}
