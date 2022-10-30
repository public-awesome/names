# Stargaze Names Deployment Scripts

## Getting started

Use an environment variable manager like [dotenv](https://github.com/motdotla/dotenv).

`cp .env.example .env` and modify for your setup.

Configure `starsd` with: `./00-config.sh`.

## Upload code to chain

Download the latest WASM code from [releases](https://github.com/public-awesome/names/releases).

`./store.sh`

Make a note of the code ids using `jq`:

```sh
starsd q tx [TX_HASH] | jq . -C | less -R
```

You can also get the code ids from the above output. Look for something like:

```json
{ "key": "code_id", "value": "245" }
```

Update `.env` with code ids.

## Instantiate Marketplace

`./init_mkt.sh`

Update `.env` with Marketplace address (`MKT`).

## Instantiate Minter + Collection

`./init_minter.sh`

Update `.env` with both the minter and collection addresses (`MINTER` and `COLLECTION`).

You can verify the correct addresses with the query helpers.

```sh
./query_col.sh
./query_minter.sh
```

Since the minter and collection addresses are output at the same time, it might be difficult to know which is which. Try one of them for `MINTER` and perform the above queries. If they fail, switch around the minter and collection.

## Setup Marketplace

Marketplace has to be setup with the minter and collection addresses.

```sh
./exec_mkt_setup.sh
```

Verify it was setup correctly with:

```sh
./query_mkt.sh
```

You should see the minter and collection addresses.

## Execute a mint

```sh
./exec_mint.sh [name]
```

## Associate name with an address

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

## Place a bid

```
./exec_bid.sh [name] [price (in STARS)]
```

## Accept a bid

```
./exec_accept_bid.sh [name] [bidder] [price (in STARS)]
```

## Deployment

All the scripts are in `/scripts`. The other relevant files are `.env` and `starsd`.
Step 0. Setup `starsd`, run `./config.sh` and set up `.env`
Follow the other steps in order of numbers.

Instantiate as many whitelists as needed. Pause minting, then add/remove whitelists as needed for the next wave. Then resume minting.
