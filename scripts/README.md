# Stargaze Names Deployment Scripts

## Getting started

Use an environment variable manager like [dotenv](https://github.com/motdotla/dotenv).

`cp .env.example .env` and modify for your setup.

Configure `starsd` with: `./setup.sh`.

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

Update `.env` with both Minter and Collection addresses (`MINTER` and `COLLECTION`).

You can verify the correct addresses with the query helpers.

```sh
./query_col.sh
./query_minter.sh
./query_mkt.sh
```

## Execute a mint

```sh
./exec_mint.sh bobo
```
