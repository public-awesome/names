import codegen from "@cosmwasm/ts-codegen";

codegen({
  contracts: [
    {
      name: "NameMarketplace",
      dir: "../contracts/marketplace/schema",
    },
    {
      name: "NameMinter",
      dir: "../contracts/name-minter/schema",
    },
    {
      name: "Sg721Name",
      dir: "../contracts/sg721-name/schema",
    },
    {
      name: "WhitelistUpdatable",
      dir: "../contracts/whitelist-updatable/schema",
    },
  ],
  outPath: "./src/",

  // options are completely optional ;)
  options: {
    bundle: {
      bundleFile: "index.ts",
      scope: "contracts",
    },
    types: {
      enabled: true,
    },
    client: {
      enabled: true,
    },
    reactQuery: {
      enabled: false,
      optionalClient: true,
      version: "v4",
      mutations: true,
      queryKeys: true,
    },
    recoil: {
      enabled: false,
    },
    messageComposer: {
      enabled: true,
    },
  },
}).then(() => {
  console.log("✨ all done!");
});
