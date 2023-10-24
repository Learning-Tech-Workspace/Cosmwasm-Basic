use cosmwasm_std::{Addr, Coin, StdResult};
use cw_multi_test::{App, ContractWrapper, Executor};

use crate::{
    error::ContractError,
    execute, instantiate,
    msg::{ExecMsg, InitMsg, QueryMsg, ValueResp},
    query,
};

pub struct CountingContract(Addr);

impl CountingContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    // like the counting_contract() in old test
    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate(
        app: &mut App,
        code_id: u64,
        sender: &Addr,
        label: &str,
        counter: impl Into<Option<u64>>, // not understand
        minimal_donation: Coin,
    ) -> StdResult<Self> {
        let counter = counter.into().unwrap_or_default(); // rust analyzer not suggest code?

        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InitMsg {
                counter,
                minimal_donation,
            },
            &[],
            label.clone(),
            None,
        )
        .map(CountingContract)
        .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn donate(
        &self,
        app: &mut App,
        sender: &Addr,
        funds: &[Coin],
    ) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Donate {}, funds)
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn reset(
        &self,
        app: &mut App,
        sender: &Addr,
        counter: impl Into<Option<u64>>,
    ) -> Result<(), ContractError> {
        let counter = counter.into().unwrap_or_default(); // basic understand mean this will set to default which 0 when we do not pass the counter
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecMsg::Reset { value: counter },
            &[],
        )
        .map_err(|err| err.downcast().unwrap())
        .map(|_| ())
    }

    #[track_caller]
    pub fn withdraw(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Withdraw {}, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn withdraw_to(
        &self,
        app: &mut App,
        sender: &Addr,
        receiver: &Addr,
        limit_funds: impl Into<Option<Vec<Coin>>>,
    ) -> Result<(), ContractError> {
        let limit_funds = limit_funds.into().unwrap_or_default();
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecMsg::WithdrawTo {
                receiver: receiver.to_string(),
                limit_funds,
            },
            &[],
        )
        .map_err(|err| err.downcast().unwrap())
        .map(|_| ())
    }

    #[track_caller]
    pub fn query_value(&self, app: &App) -> StdResult<ValueResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Value {})
    }
}

// why need this
impl From<CountingContract> for Addr {
    fn from(value: CountingContract) -> Self {
        todo!()
    }
}
