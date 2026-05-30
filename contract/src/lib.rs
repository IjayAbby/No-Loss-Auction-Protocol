#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};
use soroban_sdk::token::Client as TokenClient;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Auction(BytesN<32>),
    Refund(BytesN<32>, Address),
}

#[contracttype]
pub struct Auction {
    owner: Address,
    token: Address,
    deadline: u64,
    minimum_bid: i128,
    highest_bid: i128,
    highest_bidder: Option<Address>,
    bid_count: u32,
    active: bool,
    finalized: bool,
}

#[derive(Clone, Debug)]
pub enum AuctionError {
    AuctionAlreadyExists,
    AuctionNotFound,
    AuctionClosed,
    AuctionCanceled,
    AuctionFinalized,
    AuctionHasBids,
    BidTooLow,
    Unauthorized,
    NoBids,
    DeadlineNotReached,
    InvalidAmount,
    NoRefund,
    AuctionInactive,
}

fn require(cond: bool, err: AuctionError) {
    if !cond {
        panic!("AuctionError::{:?}", err);
    }
}

fn store_auction(env: &Env, auction_id: &BytesN<32>, auction: &Auction) {
    env.storage().persistent().set(&DataKey::Auction(auction_id.clone()), auction);
}

fn load_auction(env: &Env, auction_id: &BytesN<32>) -> Auction {
    env.storage()
        .persistent()
        .get::<DataKey, Auction>(&DataKey::Auction(auction_id.clone()))
        .unwrap_or_else(|| panic!("AuctionError::{:?}", AuctionError::AuctionNotFound))
}

fn refund_key(auction_id: &BytesN<32>, addr: &Address) -> DataKey {
    DataKey::Refund(auction_id.clone(), addr.clone())
}

fn get_refund_balance(env: &Env, auction_id: &BytesN<32>, addr: &Address) -> i128 {
    env.storage()
        .persistent()
        .get::<DataKey, i128>(&refund_key(auction_id, addr))
        .unwrap_or(0)
}

fn set_refund_balance(env: &Env, auction_id: &BytesN<32>, addr: &Address, amount: i128) {
    if amount == 0 {
        env.storage().persistent().remove(&refund_key(auction_id, addr));
    } else {
        env.storage().persistent().set(&refund_key(auction_id, addr), &amount);
    }
}

#[contract]
pub struct AuctionContract;

#[contractimpl]
impl AuctionContract {
    pub fn create_auction(
        env: Env,
        owner: Address,
        auction_id: BytesN<32>,
        token_contract: Address,
        deadline: u64,
        minimum_bid: i128,
    ) {
        owner.require_auth();
        require(deadline > env.ledger().timestamp(), AuctionError::InvalidAmount);
        require(minimum_bid > 0, AuctionError::InvalidAmount);

        let existing = env.storage().persistent().has(&DataKey::Auction(auction_id.clone()));
        require(!existing, AuctionError::AuctionAlreadyExists);

        let auction = Auction {
            owner,
            token: token_contract,
            deadline,
            minimum_bid,
            highest_bid: 0,
            highest_bidder: None,
            bid_count: 0,
            active: true,
            finalized: false,
        };
        store_auction(&env, &auction_id, &auction);
    }

    pub fn place_bid(env: Env, bidder: Address, auction_id: BytesN<32>, bid_amount: i128) {
        bidder.require_auth();
        let mut auction = load_auction(&env, &auction_id);
        require(auction.active, AuctionError::AuctionInactive);
        require(!auction.finalized, AuctionError::AuctionFinalized);
        require(env.ledger().timestamp() < auction.deadline, AuctionError::AuctionClosed);
        require(bid_amount > 0, AuctionError::InvalidAmount);
        require(bid_amount >= auction.minimum_bid && bid_amount > auction.highest_bid, AuctionError::BidTooLow);
        require(bidder != auction.owner, AuctionError::Unauthorized);

        let token_client = TokenClient::new(&env, &auction.token);
        token_client.transfer(&bidder, &env.current_contract_address(), &bid_amount);

        if let Some(prev_bidder) = auction.highest_bidder.clone() {
            let previous_amount = auction.highest_bid;
            let old_balance = get_refund_balance(&env, &auction_id, &prev_bidder);
            set_refund_balance(&env, &auction_id, &prev_bidder, old_balance + previous_amount);
        }

        auction.highest_bid = bid_amount;
        auction.highest_bidder = Some(bidder);
        auction.bid_count += 1;
        store_auction(&env, &auction_id, &auction);
    }

    pub fn claim_refund(env: Env, bidder: Address, auction_id: BytesN<32>) {
        bidder.require_auth();
        let refund_amount = get_refund_balance(&env, &auction_id, &bidder);
        require(refund_amount > 0, AuctionError::NoRefund);

        let auction = load_auction(&env, &auction_id);
        let token_client = TokenClient::new(&env, &auction.token);
        token_client.transfer(&env.current_contract_address(), &bidder, &refund_amount);
        set_refund_balance(&env, &auction_id, &bidder, 0);
    }

    pub fn finalize_auction(env: Env, caller: Address, auction_id: BytesN<32>) {
        caller.require_auth();
        let mut auction = load_auction(&env, &auction_id);
        require(auction.active, AuctionError::AuctionInactive);
        require(!auction.finalized, AuctionError::AuctionFinalized);
        require(env.ledger().timestamp() >= auction.deadline, AuctionError::DeadlineNotReached);

        auction.active = false;
        auction.finalized = true;
        store_auction(&env, &auction_id, &auction);

        if auction.bid_count > 0 {
            let token_client = TokenClient::new(&env, &auction.token);
            token_client.transfer(&env.current_contract_address(), &auction.owner, &auction.highest_bid);
        }
    }

    pub fn cancel_auction(env: Env, owner: Address, auction_id: BytesN<32>) {
        owner.require_auth();
        let mut auction = load_auction(&env, &auction_id);
        require(auction.active, AuctionError::AuctionInactive);
        require(owner == auction.owner, AuctionError::Unauthorized);
        require(auction.bid_count == 0, AuctionError::AuctionHasBids);
        require(env.ledger().timestamp() < auction.deadline, AuctionError::AuctionClosed);

        auction.active = false;
        auction.finalized = false;
        store_auction(&env, &auction_id, &auction);
    }

    pub fn auction_details(env: Env, auction_id: BytesN<32>) -> Auction {
        load_auction(&env, &auction_id)
    }

    pub fn refund_balance(env: Env, auction_id: BytesN<32>, bidder: Address) -> i128 {
        get_refund_balance(&env, &auction_id, &bidder)
    }
}
