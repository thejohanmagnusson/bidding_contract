use cw_multi_test::{Executor, ContractWrapper};
use cw_multi_test::App;
use cosmwasm_std::{Addr, Coin, StdResult, Uint128};

use crate::{execute, instantiate, query};
use crate::error::ContractError;
use crate::msg::{InstantiateMsg, ExecMsg, QueryMsg};
use crate::msg::{AuctionResp, BidResp};

pub struct BiddingContract(Addr);

impl From<BiddingContract> for Addr {
    fn from(contract: BiddingContract) -> Self {
        contract.0
    }
}

impl BiddingContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate<'a>(app: &mut App, code_id: u64, sender: &Addr, label: &str, admin: impl Into<Option<&'a Addr>>, commodity: &str, bid_asset: Coin, commission: Uint128) -> StdResult<BiddingContract> {
        let admin = admin.into();
        
        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InstantiateMsg {
                commodity: commodity.to_string(),
                bid_asset,
                commission,
                owner: admin.map(Addr::to_string),
            },
            &[],
            label,
            admin.map(Addr::to_string),
        )
        .map(BiddingContract)
        .map_err(|err| err.downcast().unwrap())
    }

    pub fn bid(&self, app: &mut App, sender: &Addr, amount: &[Coin]) -> Result<(), ContractError> {
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecMsg::Bid {},
            &amount,
        )
        .map_err(|err| err.downcast::<ContractError>().unwrap())?;

        Ok(())
    }

    pub fn close(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecMsg::Close {},
            &[],
        )
        .map_err(|err| err.downcast::<ContractError>().unwrap())?;

        Ok(())
    }

    pub fn retract(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecMsg::Retract {
                receiver: Some(sender.to_string()),
            },
            &[],
        )
        .map_err(|err| err.downcast::<ContractError>().unwrap())?;

        Ok(())
    }

    pub fn query_auction(&self, app: &App) -> StdResult<AuctionResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Auction {})
    }

    pub fn query_address(&self, app: &App, address: &Addr) -> StdResult<Coin> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Bids { address: address.to_string() })
    }

    pub fn query_highest_bid(&self, app: &App) -> StdResult<BidResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::HighestBid {})
    }
}

