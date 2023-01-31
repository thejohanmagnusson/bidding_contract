use cosmwasm_std::{Coin, DepsMut, MessageInfo, Response, StdResult, Uint128};

use crate::state::{IS_OPEN, BID_ASSET, COMMISSION, COMMODITY, OWNER};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(deps: DepsMut, info: MessageInfo, commodity: String, asset: Coin, commission: Uint128, owner: Option<String>) -> StdResult<Response> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner_addr = match owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender,
    };

    OWNER.save(deps.storage, &owner_addr)?;
    COMMODITY.save(deps.storage, &commodity)?;
    BID_ASSET.save(deps.storage, &asset)?;
    COMMISSION.save(deps.storage, &commission)?;
    IS_OPEN.save(deps.storage, &true)?;
    // No initial BIDS
    // No initial HIGEST_BID

    Ok(Response::new())
}

pub mod query {
    use cosmwasm_std::Addr;
    use cosmwasm_std::Coin;
    use cosmwasm_std::Deps;
    use cosmwasm_std::StdResult;
    use cosmwasm_std::Uint128;

    use crate::msg::{AuctionResp, BidResp};
    use crate::state::BIDS;
    use crate::state::{BID_ASSET, COMMISSION, COMMODITY, HIGEST_BID, IS_OPEN};


    pub fn auction(deps: Deps) -> StdResult<AuctionResp> {
        let commodity = COMMODITY.load(deps.storage)?;
        let bid_asset = BID_ASSET.load(deps.storage)?;
        let commission = COMMISSION.load(deps.storage)?;
        let is_open = IS_OPEN.load(deps.storage)?;

        Ok(AuctionResp {
            commodity,
            bid_asset,
            commission,
            is_open,
        })
    }

    pub fn bids(deps: Deps, address: String) -> StdResult<Coin> {
        let addr = Addr::unchecked(address);    // Ignoring to check address format as it's not critical for the contract
        let bid = BIDS.may_load(deps.storage, addr)?;

        if let Some(bid) = bid {
            return Ok(bid);
        }
        
        let bid_asset = BID_ASSET.load(deps.storage)?;

        Ok(Coin { 
            denom: bid_asset.denom, 
            amount: Uint128::new(0),
        })
    }

    pub fn highest_bid(deps: Deps) -> StdResult<BidResp> {
        let higest_bid = HIGEST_BID.may_load(deps.storage)?;

        if let Some(hb) = higest_bid {
            return Ok(BidResp {
                address: hb.address.to_string(),
                bid: hb.bid,
            })
        }

        let bid_asset = BID_ASSET.load(deps.storage)?;

        Ok(BidResp {
            address: "".to_string(),
            bid: Coin { 
                denom: bid_asset.denom, 
                amount: Uint128::new(0),
            }
        })
    }

    pub fn  winner(deps: Deps) -> StdResult<BidResp> {
        let is_open = IS_OPEN.load(deps.storage)?;
        
        if is_open {
            let bid = HIGEST_BID.may_load(deps.storage)?;

            if let Some(hb) = bid {
                return Ok(BidResp {
                    address: hb.address.to_string(),
                    bid: hb.bid,
                })
            }
        }

        let bid_asset = BID_ASSET.load(deps.storage)?;
                
        Ok(BidResp {
            address: "".to_string(),
            bid: Coin { 
                denom: bid_asset.denom, 
                amount: Uint128::new(0),
            }
        })
    }

}

pub mod exec {
    use cosmwasm_std::{BankMsg, Coin, DepsMut, MessageInfo, Response, Uint128};

    use crate::error::ContractError;
    use crate::state::{Bid, BID_ASSET, BIDS, COMMISSION, HIGEST_BID, IS_OPEN, OWNER};

    pub fn bid(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let mut resp = Response::new();

        let is_open = IS_OPEN.load(deps.storage)?;
        if !is_open {
            return Err(ContractError::BiddingClosed {});
        }

        let owner = OWNER.load(deps.storage)?;
        if info.sender == owner {
            return Err(ContractError::BiddingByOwner {});
        }

        let asset = BID_ASSET.load(deps.storage)?;
        
        match info.funds.iter().find(|c| c.denom == asset.denom) {
            Some(funds) => {
                let com_rate = COMMISSION.load(deps.storage)?;
                let com_amount = funds.amount * com_rate / Uint128::new(100);

                let bid = BIDS.may_load(deps.storage, info.sender.clone())?;
                let amount = bid.map_or(funds.amount - com_amount, |b| b.amount + funds.amount - com_amount);

                let h_bid_amount = HIGEST_BID.may_load(deps.storage)?.map(|b| b.bid.amount).unwrap_or(Uint128::new(0));

                if amount < h_bid_amount {
                    return Err(ContractError::BidToLow { higest_bid: h_bid_amount.to_string() });
                }
                BIDS.save(deps.storage, info.sender.clone(), &Coin {
                    denom: funds.denom.clone(),
                    amount,
                })?;

                // Saving the highest bid without the commission deduction
                HIGEST_BID.save(deps.storage, &Bid {
                    address: info.sender.clone(),
                    bid: Coin {
                        denom: funds.denom.clone(),
                        amount: funds.amount,
                    }
                })?;

                // Send commission to owner
                let bank_msg = BankMsg::Send {
                    to_address: owner.to_string(),
                    amount: vec![Coin {
                        denom: funds.denom.clone(),
                        amount: com_amount,
                    }],
                };
                
                resp = resp
                .add_message(bank_msg)
                .add_attribute("action", "bid")
                .add_attribute("sender", info.sender.as_str())
                .add_attribute("commission", com_amount.to_string());

                Ok(resp)
            },
            None => Err(ContractError::InvalidDenomination {denom: asset.denom}),
        }
    }

    pub fn close(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        let mut resp = Response::new();

        if info.sender != owner {
            return Err(ContractError::Unauthorized {
                owner: owner.to_string(),
            });
        }

        let is_open = IS_OPEN.load(deps.storage)?;
        if !is_open {
            return Err(ContractError::BiddingClosed {});
        }

        IS_OPEN.save(deps.storage, &false)?;
        
        let winner = HIGEST_BID.may_load(deps.storage)?;
        match winner {
            Some(winner) => {
                let funds = BIDS.load(deps.storage, winner.address.clone()).unwrap();

                let bank_msg = BankMsg::Send {
                    to_address: owner.to_string(),
                    amount: vec![funds],
                };

                resp = resp
                .add_message(bank_msg)
                .add_attribute("winner", winner.address.as_str());
            }
            None => {
                resp = resp.add_attribute("winner", "None");
            }
        }

        resp = resp
        .add_attribute("action", "close")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("bidding", "closed");

        Ok(resp)
    }

    pub fn retract(deps: DepsMut, info: MessageInfo, receiver: Option<String>) -> Result<Response, ContractError> {
        let mut resp = Response::new();

        if IS_OPEN.load(deps.storage)? {
            return Err(ContractError::BiddingOpen {});
        }

        let winner =  HIGEST_BID.may_load(deps.storage)?;
        if let Some(winner) = winner {
            if info.sender == winner.address {
                return Err(ContractError::RetractByWinner {});
            }
        }

        let addr = receiver.unwrap_or(info.sender.to_string());
        match BIDS.may_load(deps.storage, info.sender)? {
            Some(bid) => {

                let bank_msg = BankMsg::Send {
                    to_address: addr.clone(),
                    amount: vec![bid],
                };

                resp = resp
                .add_message(bank_msg)
            }
            None => {
                return Err(ContractError::NoBid {});
            }
        }

        resp = resp
        .add_attribute("action", "retract")
        .add_attribute("sender", addr);

        Ok(resp)
    }
}