use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub commodity: String,
    pub bid_asset: Coin,
    pub commission: Uint128,
    pub owner: Option<String>,
}

#[cw_serde]
pub enum ExecMsg {
    Bid {},
    Close {},
    Retract {
        receiver: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AuctionResp)]
    Auction {},
    #[returns(Coin)]
    Bids {
        address: String,
    },
    #[returns(BidResp)]
    HighestBid {},
    #[returns(BidResp)]
    Winner {},
}

#[cw_serde]
pub struct AuctionResp {
    pub commodity: String,
    pub bid_asset: Coin,
    pub commission: Uint128,
    pub is_open: bool,
}

#[cw_serde]
pub struct BidResp {
    pub address: String,
    pub bid: Coin,
}