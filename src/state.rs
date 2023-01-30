use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

pub const OWNER: Item<Addr> = Item::new("owner");
pub const COMMODITY: Item<String> = Item::new("Commodity");
pub const BID_ASSET: Item<Coin> = Item::new("bid_asset");
pub const COMMISSION: Item<Uint128> = Item::new("commission");
pub const IS_OPEN: Item<bool> = Item::new("is_open");
pub const BIDS: Map<Addr, Coin> = Map::new("bids");
pub const HIGEST_BID: Item<Bid> = Item::new("highest_bid");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Bid {
    pub address: Addr,
    pub bid: Coin,
}