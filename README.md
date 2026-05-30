# No-Loss Auction Protocol

A Soroban smart contract project for a decentralized no-loss auction system on Stellar.


## Project layout

- `contract/` — Soroban Rust smart contract source.
- `frontend/` — static web UI scaffold for auction operations.

## Live Demo

Check out live demo of this project [here.](https://loss-auction-protocol.vercel.app)

## Features

- Create auctions tied to a SEP-41 token contract
- Place bids with token transfer into contract escrow
- Track highest bidder and bid amount
- Hold losing bids as refunds until claim
- Finalize auctions after deadline, releasing proceeds to the seller
- Cancel auctions when no bids exist

## Contract

The contract is implemented in `contract/src/lib.rs`.

### Important API functions

- `create_auction(owner, auction_id, token_contract, deadline, minimum_bid)`
- `place_bid(bidder, auction_id, bid_amount)`
- `claim_refund(bidder, auction_id)`
- `finalize_auction(caller, auction_id)`
- `cancel_auction(owner, auction_id)`
- `auction_details(auction_id)`
- `refund_balance(auction_id, bidder)`

## Build

From the `contract/` directory:

```bash
cd contract
cargo check
cargo test
```

## Frontend

The UI is in `frontend/index.html` and `frontend/app.js`.

It includes:

- wallet/connect inputs
- auction creation form
- bid placement form
- finalize and cancel controls
- refund claim action

> The frontend currently uses the connected wallet address as the authenticated caller for each contract operation.

## Notes

- The contract currently stores pending refunds for outbid bidders, enabling manual refund claims.
- After deployment, set the correct contract ID and token contract ID in the frontend UI.
- The current frontend is scaffolding for Soroban invocation and can be wired to the deployed contract runtime.
