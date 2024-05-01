use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Deps, Response, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub qtum_addr: Addr,
    pub max_withdraw_qtum: Uint128,
    pub max_withdraw_inj: Uint128,
}

#[cw_serde]
pub struct State {
    pub total_testers: u64,
    pub total_qtum_claimed: Uint128,
    pub total_inj_claimed: Uint128,
}

#[cw_serde]
pub struct TesterInfo {
    pub claimed_qutm_amount: Uint128,
    pub claimed_inj_amount: Uint128,
    pub last_qtum_claimed: u64,
    pub last_inj_claimed: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const TESTER_INFO: Map<&str, TesterInfo> = Map::new("tester_info");

pub fn tester_info(deps: Deps, sender: &str) -> StdResult<TesterInfo> {
    let tester_info = TESTER_INFO.may_load(deps.storage, sender).unwrap();

    match tester_info {
        Some(tester_info) => Ok(tester_info),
        None => Ok(TesterInfo {
            claimed_qutm_amount: Uint128::zero(),
            claimed_inj_amount: Uint128::zero(),
            last_qtum_claimed: 0,
            last_inj_claimed: 0,
        }),
    }
}

pub fn store_test_user(
    storage: &mut dyn Storage,
    owner: &str,
    tester_info: &TesterInfo,
) -> StdResult<Response> {
    TESTER_INFO.save(storage, owner, tester_info)?;
    Ok(Response::new())
}

pub fn remove_test_user(storage: &mut dyn Storage, owner: &str) -> StdResult<Response> {
    TESTER_INFO.remove(storage, owner);
    Ok(Response::new())
}
