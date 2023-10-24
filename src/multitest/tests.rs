use cosmwasm_std::{coin, coins, Addr};
use cw_multi_test::{App, Executor};

use crate::{
    error::ContractError,
    msg::{ExecMsg, QueryMsg, ValueResp},
};

use super::contract::CountingContract;

const NEAR: &str = "NEAR";

#[test]
fn query_value() {
    let sender = Addr::unchecked("sender");
    let mut app = App::default();

    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Counting Contract",
        None,
        coin(10, NEAR),
    )
    .unwrap();

    let resp: ValueResp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 0);
}

#[test]
fn execute_donate() {
    let mut app = App::default();

    let code_id = CountingContract::store_code(&mut app);

    let sender = Addr::unchecked("sender");

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Counting contract",
        None,
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

    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Counting contract",
        None,
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

    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Counting contract",
        None,
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

    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Counting contract",
        None,
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

#[test]
fn query_incremented() {
    let mut app = App::default();

    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &Addr::unchecked("owner"),
        "Counting contract",
        None,
        coin(10, NEAR),
    )
    .unwrap();

    let resp: ValueResp = app
        .wrap()
        .query_wasm_smart(contract.addr(), &QueryMsg::Incremented { value: (1) })
        .unwrap();

    assert_eq!(resp, ValueResp { value: 2 });
}

#[test]
fn expecting_no_funds() {
    let sender = Addr::unchecked("sender");
    let mut app = App::default();

    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &Addr::unchecked("owner"),
        "Counting contract",
        None,
        coin(10, NEAR),
    )
    .unwrap();

    contract.donate(&mut app, &sender, &[]);

    let resp: ValueResp = contract.query_value(&app).unwrap();

    assert_eq!(resp, ValueResp { value: 0 });
}

#[test]
fn execute_reset() {
    let owner = Addr::unchecked("owner");
    let mut app = App::default();

    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &Addr::unchecked("owner"),
        "Counting contract",
        None,
        coin(10, NEAR),
    )
    .unwrap();

    let sender = Addr::unchecked("sender");

    app.execute_contract(
        owner,
        contract.addr().clone(),
        &ExecMsg::Reset { value: 5 },
        &[],
    );

    let resp: ValueResp = app
        .wrap()
        .query_wasm_smart(contract.addr(), &QueryMsg::Value {})
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

    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &Addr::unchecked("owner"),
        "Counting contract",
        None,
        coin(10, NEAR),
    )
    .unwrap();

    app.execute_contract(
        sender.clone(),
        contract.addr().clone(),
        &ExecMsg::Donate {},
        &coins(10, "NEAR"),
    )
    .unwrap();

    app.execute_contract(
        owner.clone(),
        contract.addr().clone(),
        &ExecMsg::WithdrawTo {
            receiver: (String::from("receiver")),
            limit_funds: coins(4, "NEAR"),
        },
        &[],
    )
    .unwrap();

    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
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

    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &Addr::unchecked("owner"),
        "Counting contract",
        None,
        coin(10, NEAR),
    )
    .unwrap();

    let err = app
        .execute_contract(
            receiver.clone(),
            contract.addr().clone(),
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

    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &Addr::unchecked("owner"),
        "Counting contract",
        None,
        coin(10, NEAR),
    )
    .unwrap();

    let sender = Addr::unchecked("sender");

    let err = app
        .execute_contract(
            sender.clone(),
            contract.addr().clone(),
            &ExecMsg::Reset { value: 5 },
            &[],
        )
        .unwrap_err();

    let resp: ValueResp = app
        .wrap()
        .query_wasm_smart(contract.addr(), &QueryMsg::Value {})
        .unwrap();

    assert_eq!(
        ContractError::Unauthorized {
            owner: (owner.to_string())
        },
        err.downcast().unwrap()
    )
}
