#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
use error::ContractError;

mod contract;
mod error;
pub mod msg;
#[cfg(any(test, feature = "tests"))]
pub mod multitest;
mod state;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: msg::InstantiateMsg) -> StdResult<Response> {
    contract::instantiate(deps, info, msg.commodity, msg.bid_asset, msg.commission, msg.owner)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;
    use contract::query;

    match msg {
        Auction {} => to_binary(&query::auction(deps)?),
        Bids { address } => to_binary(&query::bids(deps, address)?),
        HighestBid {} => to_binary(&query::highest_bid(deps)?),
        Winner {} => to_binary(&query::winner(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: msg::ExecMsg) -> Result<Response, ContractError> {
    use contract::exec;
    use msg::ExecMsg::*;

    match msg {
        Bid {} => exec::bid(deps, info),
        Close {} => exec::close(deps, info),
        Retract { receiver } => exec::retract(deps, info, receiver),
    }
}

