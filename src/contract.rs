use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse, TesterInfoResponse,
};
use crate::state::{store_test_user, tester_info, Config, State, TesterInfo, CONFIG, STATE};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:qtum_faucet";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: info.sender,
            qtum_addr: deps.api.addr_validate(&msg.qtum_addr)?,
            max_withdraw_qtum: msg.max_withdraw_qtum,
            max_withdraw_inj: msg.max_withdraw_inj,
        },
    )?;

    STATE.save(
        deps.storage,
        &State {
            total_testers: 0,
            total_qtum_claimed: Uint128::zero(),
            total_inj_claimed: Uint128::zero(),
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ClaimINJ { amount } => claim_inj(deps, env, info.sender, amount),
        ExecuteMsg::ClaimQtum { amount } => claim_qtum(deps, env, info.sender, amount),
    }
}

pub fn claim_qtum(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut tester_info: TesterInfo = tester_info(deps.as_ref(), sender.as_str())?;

    if tester_info.last_qtum_claimed + 86400 > env.block.time.seconds() {
        return Err(ContractError::CustomError {
            msg: "Already claimed test tokens, try after 24 hours".to_string(),
        });
    }

    if amount > config.max_withdraw_qtum {
        return Err(ContractError::CustomError {
            msg: "Exceed maxium claimable amount".to_string(),
        });
    }

    tester_info.last_qtum_claimed = env.block.time.seconds();
    tester_info.claimed_qutm_amount += amount;

    store_test_user(deps.storage, sender.as_str(), &tester_info)?;

    let claim_qtum_msg = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.qtum_addr.to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
            recipient: sender.to_string(),
            amount,
        })?,
        funds: vec![],
    })];

    Ok(Response::new()
        .add_messages(claim_qtum_msg)
        .add_attribute("action", "claim_inj")
        .add_attribute("to", sender)
        .add_attribute("amount", amount))
}

pub fn claim_inj(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut tester_info: TesterInfo = tester_info(deps.as_ref(), sender.as_str())?;

    if tester_info.last_qtum_claimed + 86400 > env.block.time.seconds() {
        return Err(ContractError::CustomError {
            msg: "Already claimed test tokens, try after 24 hours".to_string(),
        });
    }

    if amount > config.max_withdraw_inj {
        return Err(ContractError::CustomError {
            msg: "Exceed maxium claimable amount".to_string(),
        });
    }

    tester_info.last_inj_claimed = env.block.time.seconds();
    tester_info.claimed_inj_amount += amount;

    store_test_user(deps.storage, sender.as_str(), &tester_info)?;

    let claim_inj_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: sender.to_string(),
        amount: vec![Coin::new(amount.u128(), "inj")],
    });

    Ok(Response::new()
        .add_message(claim_inj_msg)
        .add_attribute("action", "claim_inj")
        .add_attribute("to", sender)
        .add_attribute("amount", amount))
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_config: Config,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::State {} => to_json_binary(&query_state(deps)?),
        QueryMsg::TesterInfo { tester } => to_json_binary(&query_staker_info(deps, tester)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    let resp = ConfigResponse {
        qtum_addr: config.qtum_addr.to_string(),
        max_withdraw_qtum: config.max_withdraw_qtum,
        max_withdraw_inj: config.max_withdraw_inj,
    };

    Ok(resp)
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;

    Ok(StateResponse {
        total_testers: state.total_testers,
        total_qtum_claimed: state.total_qtum_claimed,
        total_inj_claimed: state.total_inj_claimed,
    })
}

pub fn query_staker_info(deps: Deps, tester: String) -> StdResult<TesterInfoResponse> {
    let tester = deps.api.addr_validate(&tester)?;

    let tester_info: TesterInfo = tester_info(deps, tester.as_str())?;

    Ok(TesterInfoResponse {
        claimed_qutm_amount: tester_info.claimed_qutm_amount,
        claimed_inj_amount: tester_info.claimed_inj_amount,
        last_qtum_claimed: tester_info.last_qtum_claimed,
        last_inj_claimed: tester_info.last_inj_claimed,
    })
}
