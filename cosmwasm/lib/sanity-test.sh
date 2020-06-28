#!/bin/bash

set -euvx

function wait_for_tx () {
    until (./secretcli q tx "$1")
    do
        echo "$2"
        sleep 1
    done
}

# init the node
rm -rf ./.sgx_secrets
mkdir -p ./.sgx_secrets

rm -rf ~/.secret*
./secretcli config chain-id enigma-testnet
./secretcli config output json
./secretcli config indent true
./secretcli config trust-node true
./secretcli config keyring-backend test

./secretd init banana --chain-id enigma-testnet
perl -i -pe 's/"stake"/"uscrt"/g' ~/.secretd/config/genesis.json
echo "cost member exercise evoke isolate gift cattle move bundle assume spell face balance lesson resemble orange bench surge now unhappy potato dress number acid" |
    ./secretcli keys add a --recover
./secretd add-genesis-account "$(./secretcli keys show -a a)" 1000000000000uscrt
./secretd gentx --name a --keyring-backend test --amount 1000000uscrt
./secretd collect-gentxs
./secretd validate-genesis

./secretd init-bootstrap ./node-master-cert.der ./io-master-cert.der

./secretd validate-genesis

RUST_BACKTRACE=1 ./secretd start --bootstrap &

export secretd_PID=$(echo $!)

until (./secretcli status 2>&1 | jq -e '(.sync_info.latest_block_height | tonumber) > 0' &> /dev/null)
do
    echo "Waiting for chain to start..."
    sleep 1
done

./secretcli rest-server --chain-id enigma-testnet --laddr tcp://0.0.0.0:1337 &
export LCD_PID=$(echo $!)
function cleanup()
{
    kill -KILL "$secretd_PID" "$LCD_PID"
}
trap cleanup EXIT ERR

# store wasm code on-chain so we could later instansiate it
wget -O /tmp/contract.wasm https://raw.githubusercontent.com/CosmWasm/cosmwasm-examples/f5ea00a85247abae8f8cbcba301f94ef21c66087/erc20/contract.wasm
export STORE_TX_HASH=$(
    yes |
    ./secretcli tx compute store /tmp/contract.wasm --from a --gas 10000000 |
    jq -r .txhash
)

wait_for_tx "$STORE_TX_HASH" "Waiting for store to finish on-chain..."

# test storing of wasm code (this doesn't touch sgx yet)
./secretcli q tx "$STORE_TX_HASH" |
    jq -e '.logs[].events[].attributes[] | select(.key == "code_id" and .value == "1")'

# init the contract (ocall_init + write_db + canonicalize_address)
# a is a tendermint address (will be used in transfer: https://github.com/CosmWasm/cosmwasm-examples/blob/f5ea00a85247abae8f8cbcba301f94ef21c66087/erc20/src/contract.rs#L110)
# secret1f395p0gg67mmfd5zcqvpnp9cxnu0hg6rjep44t is just a random address
# balances are set to 108 & 53 at init
export INIT_TX_HASH=$(
    yes |
        ./secretcli tx compute instantiate 1 "{\"decimals\":10,\"initial_balances\":[{\"address\":\"$(./secretcli keys show a -a)\",\"amount\":\"108\"},{\"address\":\"secret1f395p0gg67mmfd5zcqvpnp9cxnu0hg6rjep44t\",\"amount\":\"53\"}],\"name\":\"ReuvenPersonalRustCoin\",\"symbol\":\"RPRC\"}" --label RPRCCoin --from a |
        jq -r .txhash
)

wait_for_tx "$INIT_TX_HASH" "Waiting for instantiate to finish on-chain..."

export CONTRACT_ADDRESS=$(
    ./secretcli q tx "$INIT_TX_HASH" |
        jq -er '.logs[].events[].attributes[] | select(.key == "contract_address") | .value'
)

./secretcli q compute tx "$INIT_TX_HASH"

# test balances after init (ocall_query + read_db + canonicalize_address)
./secretcli q compute contract-state smart "$CONTRACT_ADDRESS" "{\"balance\":{\"address\":\"$(./secretcli keys show a -a)\"}}" |
    jq -e '.balance == "108"'
./secretcli q compute contract-state smart "$CONTRACT_ADDRESS" "{\"balance\":{\"address\":\"secret1f395p0gg67mmfd5zcqvpnp9cxnu0hg6rjep44t\"}}" |
    jq -e '.balance == "53"'

# transfer 10 balance (ocall_handle + read_db + write_db + humanize_address + canonicalize_address)
export TRANSFER_TX_HASH=$(
    yes |
        ./secretcli tx compute execute --from a "$CONTRACT_ADDRESS" '{"transfer":{"amount":"10","recipient":"secret1f395p0gg67mmfd5zcqvpnp9cxnu0hg6rjep44t"}}' |
        jq -r .txhash
)

wait_for_tx "$TRANSFER_TX_HASH" "Waiting for transfer to finish on-chain..."

./secretcli q compute tx "$TRANSFER_TX_HASH"

# test balances after transfer (ocall_query + read_db)
./secretcli q compute contract-state smart "$CONTRACT_ADDRESS" "{\"balance\":{\"address\":\"$(./secretcli keys show a -a)\"}}" |
    jq -e '.balance == "98"'
./secretcli q compute contract-state smart "$CONTRACT_ADDRESS" "{\"balance\":{\"address\":\"secret1f395p0gg67mmfd5zcqvpnp9cxnu0hg6rjep44t\"}}" |
    jq -e '.balance == "63"'

# sleep infinity

(
    cd ./cosmwasm-js
    yarn
    cd ./packages/sdk
    yarn build
)

node ./cosmwasm/lib/cosmwasm-js-test.js

echo "All is done. Yay!"
