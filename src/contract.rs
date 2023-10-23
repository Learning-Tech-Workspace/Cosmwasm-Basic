use cosmwasm_std::{Coin, DepsMut, MessageInfo, Response, StdResult};

use crate::state::{COUNTER, MINIMAL_DONATION, OWNER};

pub fn instantiate(
    deps: DepsMut,
    info: MessageInfo,
    counter: u64,
    minimal_donation: Coin,
) -> StdResult<Response> {
    COUNTER.save(deps.storage, &counter)?;
    MINIMAL_DONATION.save(deps.storage, &minimal_donation)?;
    OWNER.save(deps.storage, &info.sender);
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
    use cosmwasm_std::{
        BankMsg, Coin, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
    };

    use crate::{
        error::ContractError,
        state::{COUNTER, MINIMAL_DONATION, OWNER},
    };

    pub fn donate(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let mut counter = COUNTER.load(deps.storage)?;
        let minimal_donation = MINIMAL_DONATION.load(deps.storage)?;

        if minimal_donation.amount.is_zero()
            || info.funds.into_iter().any(|coin| {
                coin.denom == minimal_donation.denom && coin.amount >= minimal_donation.amount
            })
        {
            counter += 1;
            COUNTER.save(deps.storage, &counter);
        };

        let resp = Response::new()
            .add_attribute("action", "donate")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", counter.to_string());

        Ok(resp)
    }

    pub fn reset(deps: DepsMut, info: MessageInfo, value: u64) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if owner != info.sender {
            return Err(ContractError::Unauthorized {
                owner: (owner.to_string()),
            });
        }

        COUNTER.update(deps.storage, |counter| -> StdResult<_> { Ok(value) })?;

        let resp = Response::new()
            .add_attribute("action", "reset")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", value.to_string());
        Ok(resp)
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if owner != info.sender {
            return Err(ContractError::Unauthorized {
                owner: (owner.clone().to_string()),
            });
        }

        let balance = deps.querier.query_all_balances(env.contract.address)?;

        let bank_msg = BankMsg::Send {
            to_address: (owner.to_string()),
            amount: (balance),
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("sender", owner.as_str());

        Ok(resp)
    }

    pub fn withdraw_to(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        receiver: String,
        limit_funds: Vec<Coin>,
    ) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {
                owner: (owner.to_string()),
            });
        }

        let checked = deps.api.addr_validate(receiver.as_str());
        if checked.is_err() {
            return Err(ContractError::InvalidAddress);
        } else {
            let receiver_addr = checked.unwrap();

            let mut balance = deps.querier.query_all_balances(env.contract.address)?;

            if !limit_funds.is_empty() {
                for coin in &mut balance {
                    let limit = limit_funds
                        .iter()
                        .find(|c| c.denom == coin.denom)
                        .map(|c| c.amount)
                        .unwrap_or(Uint128::zero());

                    coin.amount = std::cmp::min(coin.amount, limit);
                }
            }

            let bank_msg = BankMsg::Send {
                to_address: receiver_addr.to_string(),
                amount: balance,
            };

            let resp = Response::new()
                .add_message(bank_msg)
                .add_attribute("action", "withdraw_to")
                .add_attribute("sender", info.sender.as_str());
            Ok(resp)
        }
    }
}
