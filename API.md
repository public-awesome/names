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

`ewogICJhc3NvY2lhdGVkX2FkZHJlc3MiOiB7CiAgICAibmFtZSI6ICJhbGljZSIKICB9Cn0K`

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
