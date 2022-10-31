# Stargaze Names Deployment Scripts

## Requirements

- `starsd` binary installed
- Environment variable manager like [dotenv](https://github.com/motdotla/dotenv)

## Deploy Contracts

`cp .env.example .env` and modify for your setup.

### Step 1: Upload code to chain

Download the latest WASM code from [releases](https://github.com/public-awesome/names/releases).

`./01-store.sh`

Update `.env` with code ids.

### Step 2: Instantiate Marketplace

`./02-init_mkt.sh`

Update `.env` with Marketplace address (`MKT`).

### Step 3: Instantiate Minter + Collection

`./03-init_minter.sh`

Update `.env` with both the minter and collection addresses (`MINTER` and `COLLECTION`).

You can verify the correct addresses with the query helpers.

```sh
./query_col.sh
./query_minter.sh
```

Since the minter and collection addresses are output at the same time, it might be difficult to know which is which. Try one of them for `MINTER` and perform the above queries. If they fail, switch around the minter and collection.

### Step 4: Setup Marketplace

Marketplace has to be setup with the minter and collection addresses.

```sh
./04-exec_mkt_setup.sh
```

Verify it was setup correctly with:

```sh
./query_mkt.sh
```

You should see the minter and collection addresses.

## Profit!

### Execute a mint

```sh
./exec_mint.sh [name]
```

### Associate name with an address

```sh
./exec_assoc.sh [name]
```

Reverse lookup:

```sh
./query_lookup.sh
```

Query name metadata:

```sh
./query_metadata.sh [name]
```

### Place a bid

```
./exec_bid.sh [name] [price (in STARS)]
```

### Accept a bid

```
./exec_accept_bid.sh [name] [bidder] [price (in STARS)]
```

## Whitelists

Instantiate as many whitelists as needed.

Pause minting, then add/remove whitelists as needed for the next wave. Then resume minting.
