use cosmwasm_std::{Addr, Coin, coins, Uint128};
use cw_multi_test::App;

use crate::error::ContractError;
use crate::msg::{AuctionResp, BidResp};

use super::contract::BiddingContract;

const ATOM: &str = "atom";

#[test]
fn query_open_auction() {
    let sender = Addr::unchecked("sender");

    let mut app = App::default();
    let contract_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Bidding contract",
        None,
        "Item",
        Coin {
            denom: ATOM.to_string(),
            amount: Uint128::new(0),
        },
        Uint128::new(10),
    ).unwrap();

    let resp = BiddingContract::query_auction(&contract, &app).unwrap();

    assert_eq!(resp, AuctionResp {
        commodity:"Item".to_string(),
        bid_asset: Coin {
            denom: ATOM.to_string(),
            amount: Uint128::new(0),
        },
        commission: Uint128::new(10),
        is_open: true, 
    });
}

#[test]
fn query_closed_auction() {
    let sender = Addr::unchecked("sender");
    let owner = Addr::unchecked("owner");

    let mut app = App::default();
    let contract_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Bidding contract",
        &owner,
        "Item",
        Coin {
            denom: ATOM.to_string(),
            amount: Uint128::new(0),
        },
        Uint128::new(10),
    ).unwrap();

    BiddingContract::close(&contract, &mut app, &owner).unwrap();
    let resp = BiddingContract::query_auction(&contract, &app).unwrap();

    assert_eq!(resp, AuctionResp {
        commodity:"Item".to_string(),
        bid_asset: Coin {
            denom: ATOM.to_string(),
            amount: Uint128::new(0),
        },
        commission: Uint128::new(10),
        is_open: false, 
    });
}

#[test]
fn owner_can_not_bid() {
    let sender = Addr::unchecked("sender");
    let owner = Addr::unchecked("owner");

    // Setup inital funds
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &owner, coins(10, ATOM))
            .unwrap();
    });

    let contract_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Bidding contract",
        &owner,
        "Item",
        Coin {
            denom: ATOM.to_string(),
            amount: Uint128::new(0),
        },
        Uint128::new(10),
    ).unwrap();

    // Expecting error
    let err = BiddingContract::bid(&contract, &mut app, &owner, &coins(10, ATOM)).unwrap_err();

    assert_eq!(err, ContractError::BiddingByOwner {});
}

#[test]
fn bid_closed_auction() {
    let sender = Addr::unchecked("sender");
    let owner = Addr::unchecked("owner");
    let bidder = Addr::unchecked("bidder");

    // Setup inital funds
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &bidder, coins(10, ATOM))
            .unwrap();
    });

    let contract_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Bidding contract",
        &owner,
        "Item",
        Coin {
            denom: ATOM.to_string(),
            amount: Uint128::new(0),
        },
        Uint128::new(10),
    ).unwrap();

    BiddingContract::close(&contract, &mut app, &owner).unwrap();

    // Expecting error
    let err = BiddingContract::bid(&contract, &mut app, &bidder, &coins(10, ATOM)).unwrap_err();

    assert_eq!(err, ContractError::BiddingClosed {});
}

#[test]
fn query_bids_by_address() {
    let sender = Addr::unchecked("sender");
    let owner = Addr::unchecked("owner");
    let bidder = Addr::unchecked("bidder");

    // Setup inital funds
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &bidder, coins(20, ATOM))
            .unwrap();
    });

    let contract_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Bidding contract",
        &owner,
        "Item",
        Coin {
            denom: ATOM.to_string(),
            amount: Uint128::new(0),
        },
        Uint128::new(10),
    ).unwrap();

    BiddingContract::bid(&contract, &mut app, &bidder, &coins(20, ATOM)).unwrap();
    let resp = BiddingContract::query_address(&contract, &app, &bidder).unwrap();

    assert_eq!(resp, Coin {
        denom: ATOM.to_string(),
        amount: Uint128::new(18),
    });
}

#[test]
fn query_highest_bid() {
    let sender = Addr::unchecked("sender");
    let owner = Addr::unchecked("owner");
    let bidder_0 = Addr::unchecked("bidder_0");
    let bidder_1 = Addr::unchecked("bidder_1");

    // Setup inital funds
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &bidder_0, coins(10, ATOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &bidder_1, coins(20, ATOM))
            .unwrap();            
    });

    let contract_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Bidding contract",
        &owner,
        "Item",
        Coin {
            denom: ATOM.to_string(),
            amount: Uint128::new(0),
        },
        Uint128::new(10),
    ).unwrap();

    BiddingContract::bid(&contract, &mut app, &bidder_0, &coins(10, ATOM)).unwrap();
    BiddingContract::bid(&contract, &mut app, &bidder_1, &coins(20, ATOM)).unwrap();

    let resp = BiddingContract::query_highest_bid(&contract, &app).unwrap();

    assert_eq!(resp, BidResp {
        address: bidder_1.to_string(),
        bid: Coin {
            denom: ATOM.to_string(),
            amount: Uint128::new(20),
        },
    });
}

#[test]
fn retract_by_winner() {
    let sender = Addr::unchecked("sender");
    let owner = Addr::unchecked("owner");
    let bidder = Addr::unchecked("bidder");
    let winner = Addr::unchecked("winner");

    // Setup inital funds
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &bidder, coins(10, ATOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &winner, coins(20, ATOM))
            .unwrap();
    });

    let contract_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Bidding contract",
        &owner,
        "Item",
        Coin {
            denom: ATOM.to_string(),
            amount: Uint128::new(0),
        },
        Uint128::new(10),
    ).unwrap();

    BiddingContract::bid(&contract, &mut app, &bidder, &coins(10, ATOM)).unwrap();
    BiddingContract::bid(&contract, &mut app, &winner, &coins(20, ATOM)).unwrap();
    BiddingContract::close(&contract, &mut app, &owner).unwrap();
    // Expecting error
    let err = BiddingContract::retract(&contract, &mut app, &winner).unwrap_err();

    assert_eq!(err, ContractError::RetractByWinner {});
}

#[test]
fn retract_by_bidder() {
    let sender = Addr::unchecked("sender");
    let owner = Addr::unchecked("owner");
    let bidder = Addr::unchecked("bidder");
    let winner = Addr::unchecked("winner");

    // Setup inital funds
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &bidder, coins(10, ATOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &winner, coins(20, ATOM))
            .unwrap();
    });

    let contract_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Bidding contract",
        &owner,
        "Item",
        Coin {
            denom: ATOM.to_string(),
            amount: Uint128::new(0),
        },
        Uint128::new(10),
    ).unwrap();

    BiddingContract::bid(&contract, &mut app, &bidder, &coins(10, ATOM)).unwrap();
    BiddingContract::bid(&contract, &mut app, &winner, &coins(20, ATOM)).unwrap();
    BiddingContract::close(&contract, &mut app, &owner).unwrap();
    BiddingContract::retract(&contract, &mut app, &bidder).unwrap();

    assert_eq!(app.wrap().query_all_balances(bidder).unwrap(), coins(9, ATOM));
}