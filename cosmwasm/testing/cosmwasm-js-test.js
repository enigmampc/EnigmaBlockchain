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

  const tx = await client.restClient.txById(execTx.transactionHash);
  assert.deepEqual(execTx.logs, tx.logs);
  assert.deepEqual(execTx.data, tx.data);
  assert.deepEqual(tx.data, Uint8Array.from([]));
  assert.deepEqual(tx.logs[0].events[1].attributes, [
    {
      key: "contract_address",
      value: contract,
    },
    {
      key: "action",
      value: "transfer",
    },
    {
      key: "sender",
      value: "secret19hkhy46ute3g4xfr0vtcn9g6rp9824w29ut54k",
    },
    {
      key: "recipient",
      value: "secret1f395p0gg67mmfd5zcqvpnp9cxnu0hg6rjep44t",
    },
  ]);

  const qRes = await client.queryContractSmart(contract, {
    balance: { address: "secret1f395p0gg67mmfd5zcqvpnp9cxnu0hg6rjep44t" },
  });

  assert.equal(+qRes.balance, initBalance + 10);

  const qRes2 = await client.queryContractSmart(contract, {
    balance: { address: "secret19hkhy46ute3g4xfr0vtcn9g6rp9824w29ut54k" },
  });

  try {
    await signingClient.execute(contract, {
      transfer: {
        amount: "1000",
        recipient: "secret1f395p0gg67mmfd5zcqvpnp9cxnu0hg6rjep44t",
      },
    });
  } catch (err) {
    assert(
      err.message.includes(
        `Insufficient funds: balance=${qRes2.balance}, required=1000"`
      )
    );

    const txId = /Error when posting tx (.+?)\./.exec(err.message)[1];

    const tx = await client.restClient.txById(txId);
    assert(
      tx.raw_log.includes(
        `Insufficient funds: balance=${qRes2.balance}, required=1000"`
      )
    );
  }

  try {
    await client.queryContractSmart(contract, {
      balance: { address: "blabla" },
    });
  } catch (err) {
    assert(err.message.includes("canonicalize_address returned error"));
  }

  console.log("ok 👌");
})();
