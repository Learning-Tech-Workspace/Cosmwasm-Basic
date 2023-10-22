use cosmwasm_schema::write_api;
use counting_contract::msg::{ExecMsg, InitMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InitMsg,
        execute: ExecMsg,
        query: QueryMsg,
    }
}
