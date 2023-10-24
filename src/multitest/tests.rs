use cosmwasm_std::{coin, coins, Addr, Empty};
use cw_multi_test::{App, Contract, ContractWrapper};

use crate::{
    contract,
    error::ContractError,
    execute, instantiate,
    msg::{QueryMsg, ValueResp},
    multitest::CountingContract,
    query,
};

fn counting_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}

const NEAR: &str = "NEAR";

#[test]
fn query_value() {
    let sender = Addr::unchecked("sender");
    let mut app = App::default(); // like the stimulation blockchain

    let contract_id = app.store_code(counting_contract()); // like the deploy smart contract into blockchain

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Counting Contract",
        coin(10, NEAR),
    )
    .unwrap();

    let resp: ValueResp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 0);
}

#[test]
fn execute_donate() {
    let mut app = App::default();

    let contract_id = app.store_code(counting_contract());

    let sender = Addr::unchecked("sender");

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Counting contract",
        coin(10, NEAR),
    )
    .unwrap();

    contract.donate(&mut app, &sender, &[]);

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 0);
}

#[test]
fn execute_donate_with_funds() {
    let sender = Addr::unchecked("sender");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, "NEAR"))
            .unwrap()
    });

    let contract_id = app.store_code(counting_contract());

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Counting contract",
        coin(10, NEAR),
    )
    .unwrap();

    contract.donate(&mut app, &sender, &coins(10, "NEAR"));

    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp.value, 1);

    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), &[]);
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(10, NEAR)
    );
}

#[test]
fn execute_withdraw() {
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(10, "NEAR"))
            .unwrap();

        router
            .bank
            .init_balance(storage, &sender2, coins(5, "NEAR"))
            .unwrap()
    });

    let contract_id = app.store_code(counting_contract());

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        "Counting contract",
        coin(10, NEAR),
    )
    .unwrap();

    contract.donate(&mut app, &sender1, &coins(10, "NEAR"));

    contract.donate(&mut app, &sender2, &coins(5, "NEAR"));

    contract.withdraw(&mut app, &owner);

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(
        app.wrap().query_all_balances(owner).unwrap(),
        coins(15, "NEAR")
    );

    assert_eq!(app.wrap().query_all_balances(sender1).unwrap(), &[]);
    assert_eq!(app.wrap().query_all_balances(sender2).unwrap(), &[]);
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        vec![]
    );
}

#[test]
fn execute_unauthorized_withdraw() {
    let owner = Addr::unchecked("owner");
    let member = Addr::unchecked("member");

    let mut app = App::default();

    let contract_id = app.store_code(counting_contract());

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        "Counting contract",
        coin(10, NEAR),
    )
    .unwrap();

    let err = contract.withdraw(&mut app, &member).unwrap_err();

    assert_eq!(
        ContractError::Unauthorized {
            owner: owner.to_string()
        },
        err,
    );
}
