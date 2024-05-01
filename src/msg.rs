use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub qtum_addr: String,
    pub max_withdraw_qtum: Uint128,
    pub max_withdraw_inj: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    ClaimQtum { amount: Uint128 },
    ClaimINJ { amount: Uint128 },
}

// query msgs

#[cw_serde]
pub enum QueryMsg {
    Config {},
    State {},
    TesterInfo { tester: String },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub qtum_addr: String,
    pub max_withdraw_qtum: Uint128,
    pub max_withdraw_inj: Uint128,
}

#[cw_serde]
pub struct StateResponse {
    pub total_testers: u64,
    pub total_qtum_claimed: Uint128,
    pub total_inj_claimed: Uint128,
}

#[cw_serde]
pub struct TesterInfoResponse {
    pub claimed_qutm_amount: Uint128,
    pub claimed_inj_amount: Uint128,
    pub last_qtum_claimed: u64,
    pub last_inj_claimed: u64,
}
