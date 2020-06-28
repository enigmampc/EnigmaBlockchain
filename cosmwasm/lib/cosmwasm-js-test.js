#!/usr/bin/env node
const path = require("path");
const cosmwasmjs = require(path.resolve(
  __dirname,
  "../../cosmwasm-js/packages/sdk/build/"
));
const assert = require("assert").strict;

(async () => {
  const client = new cosmwasmjs.CosmWasmClient("http://localhost:1337");
  const contract = (await client.getContracts(1))[0].address;

  const resQuery = await client.queryContractSmart(contract, {
    balance: { address: "secret1f395p0gg67mmfd5zcqvpnp9cxnu0hg6rjep44t" },
  });
  const initBalance = +resQuery.balance;
  console.log(`js: initBalance is ${initBalance}`);

  const pen = await cosmwasmjs.Secp256k1Pen.fromMnemonic(
    "cost member exercise evoke isolate gift cattle move bundle assume spell face balance lesson resemble orange bench surge now unhappy potato dress number acid"
  );
  const address = cosmwasmjs.pubkeyToAddress(
    cosmwasmjs.encodeSecp256k1Pubkey(pen.pubkey),
    "secret"
  );
  const signingClient = new cosmwasmjs.SigningCosmWasmClient(
    "http://localhost:1337",
    address,
    (signBytes) => pen.sign(signBytes),
    {
      upload: {
        amount: [{ amount: "25000", denom: "uscrt" }],
        gas: "1000000",
      },
      init: {
        amount: [{ amount: "12500", denom: "uscrt" }],
        gas: "500000",
      },
      exec: {
        amount: [{ amount: "5000", denom: "uscrt" }],
        gas: "200000",
      },
      send: {
        amount: [{ amount: "2000", denom: "uscrt" }],
        gas: "80000",
      },
    }
  );

  const execTx = await signingClient.execute(contract, {
    transfer: {
      amount: "10",
      recipient: "secret1f395p0gg67mmfd5zcqvpnp9cxnu0hg6rjep44t",
    },
  });

  const res2Query = await client.queryContractSmart(contract, {
    balance: { address: "secret1f395p0gg67mmfd5zcqvpnp9cxnu0hg6rjep44t" },
  });

  // const tx = await client.restClient.txById(execTx.transactionHash);

  console.log(
    `js: finalBalance is ${res2Query.balance} (should be ${initBalance + 10})`
  );
  assert.equal(+res2Query.balance, initBalance + 10);
})();
