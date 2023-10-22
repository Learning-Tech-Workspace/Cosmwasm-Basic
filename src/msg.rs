use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ValueResp)]
    Value {},
    #[returns(ValueResp)]
    Incremented { value: u64 },
}

#[cw_serde]
pub struct ValueResp {
    pub value: u64,
}

#[cw_serde]
pub struct InitMsg {
    #[serde(default)]
    pub counter: u64,
    pub minimal_donation: Coin,
}

#[cw_serde]
pub enum ExecMsg {
    Donate {},
    Reset {
        value: u64,
    },
    Withdraw {},
    WithdrawTo {
        receiver: String,
        limit_funds: Vec<Coin>,
    },
}
