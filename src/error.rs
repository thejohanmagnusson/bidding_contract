use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}.")]
    Std(#[from] StdError),

    #[error("Unauthorized - only {owner} can call it.")]
    Unauthorized { owner: String },

    #[error("Bidding is closed.")]
    BiddingClosed {},

    #[error("Bidding is still open.")]
    BiddingOpen {},

    #[error("Owner can not bid.")]
    BiddingByOwner { },

    #[error("Bidds must be in {denom}.")]
    InvalidDenomination { denom: String },

    #[error("Bid is to low, current highest bid is {higest_bid}.")]
    BidToLow { higest_bid: String },

    #[error("Winner can not retract funds.")]
    RetractByWinner {},

    #[error("No placed bids.")]
    NoBid {},
}