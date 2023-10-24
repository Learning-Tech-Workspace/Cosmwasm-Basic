use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

use error::ContractError;
use msg::{ExecMsg, InitMsg, QueryMsg};

mod contract;
pub mod error;
pub mod msg;
#[cfg(test)]
mod multitest;
mod state;

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

#[cfg(test)]
mod test {
    use cosmwasm_std::{coin, coins, Addr, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    use crate::{
        error::ContractError,
        execute, instantiate,
        msg::{ExecMsg, InitMsg, QueryMsg, ValueResp},
        query,
    };

    // forward message to correct entry point
    // think simple it is the contract
    fn counting_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    #[test]
    fn query_incremented() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &InitMsg {
                    counter: 10,
                    minimal_donation: coin(10, "NEAR"),
                },
                &[],
                "Counting Contract",
                None,
            )
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Incremented { value: (1) })
            .unwrap();

        assert_eq!(resp, ValueResp { value: 2 });
    }

    #[test]
    fn expecting_no_funds() {
        let sender = Addr::unchecked("sender");
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender.clone(),
                &InitMsg {
                    counter: 0,
                    minimal_donation: coin(0, "Bitcoin"),
                },
                &[],
                "Counting Contract",
                None,
            )
            .unwrap();

        app.execute_contract(sender, contract_addr.clone(), &ExecMsg::Donate {}, &[])
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 1 });
    }

    #[test]
    fn execute_reset() {
        let owner = Addr::unchecked("owner");
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let sender = Addr::unchecked("sender");

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InitMsg {
                    counter: 0,
                    minimal_donation: coin(10, "NEAR"),
                },
                &[],
                "Counting Contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            owner,
            contract_addr.clone(),
            &ExecMsg::Reset { value: 5 },
            &[],
        );

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 5 });
    }

    #[test]
    fn execute_withdraw_to() {
        let owner = Addr::unchecked("owner");
        let sender = Addr::unchecked("sender");
        let receiver = Addr::unchecked("receiver");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(10, "NEAR"))
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InitMsg {
                    counter: 0,
                    minimal_donation: coin(5, "NEAR"),
                },
                &[],
                "Counting Contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            sender.clone(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &coins(10, "NEAR"),
        )
        .unwrap();

        app.execute_contract(
            owner.clone(),
            contract_addr.clone(),
            &ExecMsg::WithdrawTo {
                receiver: (String::from("receiver")),
                limit_funds: coins(4, "NEAR"),
            },
            &[],
        )
        .unwrap();

        assert_eq!(
            app.wrap().query_all_balances(contract_addr).unwrap(),
            coins(6, "NEAR")
        );
        assert_eq!(app.wrap().query_all_balances(sender).unwrap(), &[]);
        assert_eq!(
            app.wrap().query_all_balances(receiver).unwrap(),
            coins(4, "NEAR")
        );
    }

    #[test]
    fn execute_unauthorized_withdraw_to() {
        let owner = Addr::unchecked("owner");
        let receiver = Addr::unchecked("receiver");

        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InitMsg {
                    counter: 0,
                    minimal_donation: coin(5, "NEAR"),
                },
                &[],
                "Counting Contract",
                None,
            )
            .unwrap();

        let err = app
            .execute_contract(
                receiver.clone(),
                contract_addr.clone(),
                &ExecMsg::WithdrawTo {
                    receiver: (String::from("receiver")),
                    limit_funds: coins(4, "NEAR"),
                },
                &[],
            )
            .unwrap_err();

        assert_eq!(
            ContractError::Unauthorized {
                owner: (owner.to_string())
            },
            err.downcast().unwrap()
        )
    }

    #[test]
    fn execute_unauthorized_reset() {
        let owner = Addr::unchecked("owner");
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let sender = Addr::unchecked("sender");

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InitMsg {
                    counter: 0,
                    minimal_donation: coin(10, "NEAR"),
                },
                &[],
                "Counting Contract",
                None,
            )
            .unwrap();

        let err = app
            .execute_contract(
                sender.clone(),
                contract_addr.clone(),
                &ExecMsg::Reset { value: 5 },
                &[],
            )
            .unwrap_err();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(
            ContractError::Unauthorized {
                owner: (owner.to_string())
            },
            err.downcast().unwrap()
        )
    }
}
