use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

use msg::{ExecMsg, InitMsg, QueryMsg};

mod contract;
pub mod msg;
mod state;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    // like the main function
    // call when first time create
    use contract::instantiate;
    instantiate(deps, msg.counter);
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
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecMsg) -> StdResult<Response> {
    use contract::execute::{poke, reset};
    use msg::ExecMsg::*;
    match msg {
        Poke {} => poke(deps, info),
        Reset { value } => reset(deps, info, value),
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{Addr, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    use crate::{
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
    fn query_value() {
        let mut app = App::default(); // like the stimulation blockchain

        let contract_id = app.store_code(counting_contract()); // like the deploy smart contract into blockchain

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &InitMsg { counter: 10 },
                &[],
                "Counting contract",
                None,
            )
            .unwrap(); // like run constructor method of contract

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap(); // like call method on contract and get response

        assert_eq!(resp, ValueResp { value: 10 });
    }

    #[test]
    fn query_incremented() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &InitMsg { counter: 10 },
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
    fn execute_poke() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let sender = Addr::unchecked("sender");

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender.clone(),
                &InitMsg { counter: 0 },
                &[],
                "Counting Contract",
                None,
            )
            .unwrap();

        // increment counter in state by one
        app.execute_contract(sender, contract_addr.clone(), &ExecMsg::Poke {}, &[]);

        // get counter from state
        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 1 });
    }

    #[test]
    fn execute_reset() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let sender = Addr::unchecked("sender");

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender.clone(),
                &InitMsg { counter: 0 },
                &[],
                "Counting Contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            sender,
            contract_addr.clone(),
            &ExecMsg::Reset { value: 5 },
            &[],
        );

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 1 });
    }
}
