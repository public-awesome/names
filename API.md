# Stargaze Names API Docs

Stargaze Names associates human-readable usernames with Cosmos addresses. Address lookups can be done via an API or Typescript library.

## API

### Variables

| Network | `endpoint`                         | `contract`                                                         |
| ------- | ---------------------------------- | ------------------------------------------------------------------ |
| Testnet | `rest.elgafar-1.stargaze-apis.com` | `stars1rgn9tuxnl3ju9td3mfxdl2vm4t8xuaztcdakgtyx23c4ffm97cus25fvjs` |
| Mainnet | `rest.stargaze-apis.com`           | `stars1fx74nkqkw2748av8j7ew7r3xt9cgjqduwn8m0ur5lhe49uhlsasszc5fhr` |

### Query Associated Address

Given a name, get its associated address. Queries are base64 encoded.

Let's say you want to query the name `alice`.

```json
{
  "associated_address": {
    "name": "alice"
  }
}
```

`query`:

`ewogICJhc3NvY2lhdGVkX2FkZHJlc3MiOiB7CiAgICAibmFtZSI6ICJhbGljZSIKICB9Cn0=`

API call:

```
{endpoint}/cosmwasm/wasm/v1/contract/{contract}/smart/{query}
```

### Query Name

Given an address, query it's associated name. An address can be _any_ Cosmos address for a chain that uses the 118 coin type. In the future, Stargaze Names will support other coin types.

```json
{
  "name": { "address": "stars1tqzzmxsvzu4952mnd5ul800wfusr6p72magyge" }
}
```

`query`:

`ewogICJuYW1lIjogeyAiYWRkcmVzcyI6ICJzdGFyczF0cXp6bXhzdnp1NDk1Mm1uZDV1bDgwMHdmdXNyNnA3Mm1hZ3lnZSIgfQp9Cg==`

API call:

```
{endpoint}/cosmwasm/wasm/v1/contract/{contract}/smart/{query}
```

## Typescript

### Variables

| Network | `endpoint`                        | `contract`                                                         |
| ------- | --------------------------------- | ------------------------------------------------------------------ |
| Testnet | `rpc.elgafar-1.stargaze-apis.com` | `stars1rgn9tuxnl3ju9td3mfxdl2vm4t8xuaztcdakgtyx23c4ffm97cus25fvjs` |
| Mainnet | `rpc.stargaze-apis.com`           | `stars1fx74nkqkw2748av8j7ew7r3xt9cgjqduwn8m0ur5lhe49uhlsasszc5fhr` |

### Query Associated Address

```ts
import { CosmWasmClient } from "cosmwasm";

const client = await CosmWasmClient.connect(endpoint);

const address = await client.queryContractSmart(contract, {
  associated_address: { name: "alice" },
});

console.log("address:", address);
```

### Query Name

```ts
import { CosmWasmClient } from "cosmwasm";

const client = await CosmWasmClient.connect(endpoint);

const address = await client.queryContractSmart(contract, {
  name: { address: "stars1tqzzmxsvzu4952mnd5ul800wfusr6p72magyge" },
});

console.log("name:", name);
```
