# Casper Account Info Contract

Casper Account Info Contract allows account owners to provide information about themselves to the public by specifying a URL to a [Casper Account Info Standard](https://github.com/make-software/casper-account-info-standard) file.   

## Table of contents

- [Interacting with the contract](#interacting-with-the-contract)
  - [Known contract hashes](#known-contract-hashes)
  - [As Casper account owner](#as-casper-account-owner)
    - [Set URL for your account](#set-url-for-your-account)
    - [Delete the URL previously set for your account](#delete-the-url-previously-set-for-your-account)
    - [Get the URL set for an account](#get-the-url-set-for-an-account)
      - [Using the ```tools/get-account-info-url.sh``` and ```tools/get-account-info.sh``` scripts](#using-the-toolsget-account-info-urlsh-and-toolsget-account-infosh-scripts)
  - [As the contract admin](#as-the-contract-admin)
    - [Set URL for account](#set-url-for-account)
    - [Delete URL for account](#delete-url-for-account)
    - [Add account as an admin](#add-account-as-an-admin)
    - [Disable admin account](#disable-admin-account)
    - [Set amount of CSPR to burn during the first ```set_url``` call](#set-amount-of-cspr-to-burn-during-the-first-set_url-call)
    - [Check if account is an admin](#check-if-account-is-an-admin)
        - [Using the ```tools/is-admin.sh``` script](#using-the-toolsis-adminsh-script)
    - [Get the amount of CSPR that should be burned on the first ```set_url``` call](#get-the-amount-of-cspr-that-should-be-burned-on-the-first-set_url-call)
        - [Using the ```tools/get-cspr-to-burn-value.sh``` script](#using-the-toolsget-cspr-to-burn-valuesh-script)
- [Contract deployment](#contract-deployment)
- [Contract API](#contract-api)
  - [Public entry points](#public-entry-points)
    - [```set_url```](#set_url)
    - [```get_url```](#get_url)
    - [```delete_url```](#delete_url)
  - [Admin entry points](#admin-entry-points)
    - [```set_url_for_account```](#set_url_for_account)
    - [```delete_url_for_account```](#delete_url_for_account)
    - [```add_admin```](#add_admin)
    - [```disable_admin```](#disable_admin)
    - [```set_cspr_to_burn```](#set_cspr_to_burn)
- [Development](#development)
  - [Setup](#setup)
  - [Build](#build)
  - [Test](#test)

## Interacting with the contract

You need to have ```casper-client``` and ```jq``` installed on your system to run the examples. The instructions have been tested on Ubuntu 20.04.2 LTS.

### Install the prerequisites

You can install the required software by issuing the following commands. If you are on an up-to-date Casper node, you probably already have all of the prerequisites installed so you can skip this step.

```bash
# Update package repositories
sudo apt update

# Install the command-line JSON processor
sudo apt install jq -y

# Add Casper repository
echo "deb https://repo.casperlabs.io/releases" focal main | sudo tee -a /etc/apt/sources.list.d/casper.list
curl -O https://repo.casperlabs.io/casper-repo-pubkey.asc
sudo apt-key add casper-repo-pubkey.asc
sudo apt update

# Install the Casper client software
sudo apt install casper-client -y
```

### Known contract hashes

To interact with the contract you need to call it by its hash. The table below contains the known contract hashes (without the ```hash-``` prefix) on public Casper networks:

Network | Account info contract hash | Contract owner
---|---|---
Testnet | ```2f36a35edcbaabe17aba805e3fae42699a2bb80c2e0c15189756fdc4895356f8``` | Make Software
Mainnet | ```fb8e0215c040691e9bbe945dd22a00989b532b9c2521582538edb95b61156698``` | Casper Association

Network | Json File Name |
---|---|
Testnet | ```account-info.casper-test.json``` |
Mainnet | ```account-info.casper.json``` |

### As Casper account owner

Please set the following environment variables, which are reused across the examples:

```bash
NODE_ADDRESS=127.0.0.1
CHAIN_NAME=$(curl -s http://$NODE_ADDRESS:8888/status | jq -r '.chainspec_name')
ACCOUNT_KEYS_PATH=/etc/casper/validator_keys
ACCOUNT_INFO_CONTRACT_HASH=2f36a35edcbaabe17aba805e3fae42699a2bb80c2e0c15189756fdc4895356f8
```

The values provided above assume that you are running commands on your Testnet validator node. If you are on your Mainnet validator node, please adjust the value of the `ACCOUNT_INFO_CONTRACT_HASH` to match the Mainnet contract.

#### Set URL for your account

> **Payment:** The ```set_url``` entry point call payment should be 10 CSPR. The deploy may fail with an "Out of gas" error if a smaller amount provided. For the consecutive ```set_url``` calls the advised payment amount is 0.5 CSPR

The command below sets the top level domain URL for an account information file hosted at [https://casper-account-info-example.make.services/.well-known/casper/account-info.casper-test.json](https://casper-account-info-example.make.services/.well-known/casper/account-info.casper-test.json). (Please note that the url you provide here will be prominently displayed on the block explorers as your official website, and your account's public key must exist in the JSON data either in the nodes or affiliated accounts section for it to be successfully verified by CSPR.live and other dApps in the Casper ecosystem.)

```bash
sudo -u casper casper-client put-deploy \
    --chain-name "$CHAIN_NAME" \
    --node-address "http://$NODE_ADDRESS:7777/" \
    --secret-key "$ACCOUNT_KEYS_PATH/secret_key.pem" \
    --session-hash "$ACCOUNT_INFO_CONTRACT_HASH" \
    --session-entry-point "set_url" \
    --payment-amount 10000000000 \
    --session-arg=url:"string='https://casper-account-info-example.make.services'"
```

#### Delete the URL previously set for your account

> **Payment:** The advised payments for the ```delete_url``` entry point call is 0.5 CSPR. The deploy may fail with an "Out of gas" error if a smaller amount provided.

```bash
sudo -u casper casper-client put-deploy \
    --chain-name "$CHAIN_NAME" \
    --node-address "http://$NODE_ADDRESS:7777/" \
    --secret-key "$ACCOUNT_KEYS_PATH/secret_key.pem" \
    --session-hash "$ACCOUNT_INFO_CONTRACT_HASH" \
    --session-entry-point "delete_url" \
    --payment-amount 500000000
```

#### Get the URL set for an account

##### Using ```casper-client```

The URL string can be received by querying the ```account-info-urls``` dictionary key named as the corresponding account hash, which can be accessed by URef with the same name stored under the contract named keys.

###### Get the ```STATE_ROOT_HASH``` value

```bash
STATE_ROOT_HASH=$(casper-client get-state-root-hash --node-address http://$NODE_ADDRESS:7777 | jq -r '.result | .state_root_hash')
```

###### Get the ```account-info-contract-url``` dictionary URef

```bash
ACCOUNT_INFO_URLS_DICT_UREF=$(casper-client query-state \
  --node-address http://$NODE_ADDRESS:7777 \
  --state-root-hash "$STATE_ROOT_HASH" \
  --key "hash-$ACCOUNT_INFO_CONTRACT_HASH" \
| jq -rc '.result | .stored_value | .Contract | .named_keys | map(select(.name | contains("account-info-urls"))) | .[] .key')
```

###### Query the dictionary 

```bash
PUBLIC_KEY=<put here the public key you want to query URL for>

casper-client get-dictionary-item \
  --node-address http://$NODE_ADDRESS:7777 \
  --state-root-hash "$STATE_ROOT_HASH" \
  --seed-uref  "$ACCOUNT_INFO_URLS_DICT_UREF" \
  --dictionary-item-key "$(casper-client account-address -public-key $PUBLIC_KEY | sed -r 's/account-hash-//g')"
```

##### Using the ```tools/get-account-info-url.sh``` and ```tools/get-account-info.sh``` scripts

###### Get the URL

```bash
./get-account-info-url.sh --node-address=$NODE_ADDRESS --contract-hash=$ACCOUNT_INFO_CONTRACT_HASH --public-key=$PUBLIC_KEY
```

###### Get the account information file content

```bash
./get-account-info.sh --node-address=$NODE_ADDRESS --contract-hash=$ACCOUNT_INFO_CONTRACT_HASH --public-key=$PUBLIC_KEY
```

You can query the specific fields with ```jq```. The command below print the public key owner name:

```bash
./get-account-info.sh --node-address=$NODE_ADDRESS --contract-hash=$ACCOUNT_INFO_CONTRACT_HASH --public-key=$PUBLIC_KEY | jq -r '.owner | .name'
```

### As the contract admin 

Please set the following environment variables, which are reused across the examples:

```bash
NODE_ADDRESS=127.0.0.1
CHAIN_NAME=$(curl -s http://$NODE_ADDRESS:8888/status | jq -r '.chainspec_name')
CONTRACT_OWNER_KEYS_PATH=/etc/casper/validator_keys
ACCOUNT_INFO_CONTRACT_HASH=2f36a35edcbaabe17aba805e3fae42699a2bb80c2e0c15189756fdc4895356f8
```

The values provided above assume that you are running commands on your Testnet node.

#### Set URL for account

> **Payment:** The advised payments for the ```set_url_for_account``` entry point call is 0.5 CSPR. The deploy may fail with an "Out of gas" error if a smaller amount provided.

```
PUBLIC_KEY=<put here the public key you want to set URL for>

casper-client put-deploy \
    --chain-name "$CHAIN_NAME" \
    --node-address "http://$NODE_ADDRESS:7777/" \
    --secret-key "$CONTRACT_OWNER_KEYS_PATH/secret_key.pem" \
    --session-hash "$ACCOUNT_INFO_CONTRACT_HASH" \
    --session-entry-point "set_url_for_account" \
    --payment-amount 500000000 \
    --session-arg=account:"account_hash='$(casper-client account-address -public-key $PUBLIC_KEY)'" \
    --session-arg=url:"string='https://casper-account-info-example.make.services'"
```

#### Delete URL for account

> **Payment:** The advised payments for the ```delete_url_for_account``` entry point call is 0.5 CSPR. The deploy may fail with an "Out of gas" error if a smaller amount provided.

```
PUBLIC_KEY=<put here the public key you want to delete URL for>

casper-client put-deploy \
    --chain-name "$CHAIN_NAME" \
    --node-address "http://$NODE_ADDRESS:7777/" \
    --secret-key "$CONTRACT_OWNER_KEYS_PATH/secret_key.pem" \
    --session-hash "$ACCOUNT_INFO_CONTRACT_HASH" \
    --session-entry-point "delete_url_for_account" \
    --payment-amount 500000000 \
    --session-arg=account:"account_hash='$(casper-client account-address --public-key $PUBLIC_KEY)'"
```

#### Add account as an admin

> **Payment:** The advised payments for the ```add_admin``` entry point call is 0.5 CSPR. The deploy may fail with an "Out of gas" error if a smaller amount provided.

```
PUBLIC_KEY=<put here the public key of the account your want to make an admin>

casper-client put-deploy \
    --chain-name "$CHAIN_NAME" \
    --node-address "http://$NODE_ADDRESS:7777/" \
    --secret-key "$CONTRACT_OWNER_KEYS_PATH/secret_key.pem" \
    --session-hash "$ACCOUNT_INFO_CONTRACT_HASH" \
    --session-entry-point "add_admin" \
    --payment-amount 500000000 \
    --session-arg=account:"account_hash='$(casper-client account-address --public-key $PUBLIC_KEY)'"
```

#### Disable admin account

> **Payment:** The advised payments for the ```disable_admin``` entry point call is 0.5 CSPR. The deploy may fail with an "Out of gas" error if a smaller amount provided.

```
PUBLIC_KEY=<put here the public key of the admin account your want to disable>

casper-client put-deploy \
    --chain-name "$CHAIN_NAME" \
    --node-address "http://$NODE_ADDRESS:7777/" \
    --secret-key "$CONTRACT_OWNER_KEYS_PATH/secret_key.pem" \
    --session-hash "$ACCOUNT_INFO_CONTRACT_HASH" \
    --session-entry-point "disable_admin" \
    --payment-amount 500000000 \
    --session-arg=account:"account_hash='$(casper-client account-address -public-key $PUBLIC_KEY)'"
```

#### Set amount of CSPR to burn during the first ```set_url``` call

To avoid spamming the contract with URL entries the first ```set_url``` for an account will burn an amount of CSPR specified in the contract configuration (default is 9 CSPR). This entry point changes that amount.

> **Payment:** The advised payments for the ```set_cspr_to_burn``` entry point call is 0.5 CSPR. The deploy may fail with an "Out of gas" error if a smaller amount provided.

```
PUBLIC_KEY=<put here the public key of the admin account your want to disable>

casper-client put-deploy \
    --chain-name "$CHAIN_NAME" \
    --node-address "http://$NODE_ADDRESS:7777/" \
    --secret-key "$CONTRACT_OWNER_KEYS_PATH/secret_key.pem" \
    --session-hash "$ACCOUNT_INFO_CONTRACT_HASH" \
    --session-entry-point "set_cspr_to_burn" \
    --payment-amount 500000000 \
    --session-arg=cspr_to_burn:"u32='9'"
```

#### Check if account is an admin

##### Using ```casper-client```

The URL string can be received by querying the ```account-info-admins``` dictionary key named as the corresponding account hash, which can be accessed by URef with the same name stored under the contract named keys.

###### Get the ```account-info-contract-url``` dictionary URef

```bash
ACCOUNT_INFO_ADMINS_DICT_UREF=$(casper-client query-state \
  --node-address http://$NODE_ADDRESS:7777 \
  --state-root-hash "$STATE_ROOT_HASH" \
  --key "hash-$ACCOUNT_INFO_CONTRACT_HASH" \
| jq -rc '.result | .stored_value | .Contract | .named_keys | map(select(.name | contains("account-info-admins"))) | .[] .key')
```

###### Query the dictionary

```bash
PUBLIC_KEY=<the public key of the account you want to check>

casper-client get-dictionary-item \
  --node-address http://$NODE_ADDRESS:7777 \
  --state-root-hash "$STATE_ROOT_HASH" \
  --seed-uref  "$ACCOUNT_INFO_ADMINS_DICT_UREF" \
  --dictionary-item-key "$(casper-client account-address -public-key $PUBLIC_KEY | sed -r 's/account-hash-//g')"
```

##### Using the ```tools/is-admin.sh``` script

```bash
./is-admin.sh --node-address=$NODE_ADDRESS --contract-hash=$ACCOUNT_INFO_CONTRACT_HASH --public-key=$PUBLIC_KEY
```

#### Get the amount of CSPR that should be burned on the first ```set_url``` call

##### Using ```casper-client```

The URL string can be received by querying the ```cspr_to_burn``` value, which can be accessed by URef with the same name stored under the contract named keys.

###### Get the ```account-info-contract-url``` dictionary URef

```bash
CSPR_TO_BURN_VALUE_UREF=$(casper-client query-state \
  --node-address http://$NODE_ADDRESS:7777 \
  --state-root-hash "$STATE_ROOT_HASH" \
  --key "hash-$ACCOUNT_INFO_CONTRACT_HASH" \
| jq -rc '.result | .stored_value | .Contract | .named_keys | map(select(.name | contains("cspr_to_burn"))) | .[] .key')
```

###### Query the network

```bash
STATE_ROOT_HASH=$(casper-client get-state-root-hash --node-address http://$NODE_ADDRESS:7777 | jq -r '.result | .state_root_hash')

casper-client query-state --node-address http://$NODE_ADDRESS:7777 --key "$CSPR_TO_BURN_VALUE_UREF" --state-root-hash "$STATE_ROOT_HASH" | jq -r '.result | .stored_value'
```

##### Using the ```tools/get-cspr-to-burn-value.sh``` script

```bash
./get-cspr-to-burn-value.sh --node-address=$NODE_ADDRESS --contract-hash=$ACCOUNT_INFO_CONTRACT_HASH
```

## Contract deployment

See Casper documentation about [Deploying Contracts](https://docs.casperlabs.io/en/latest/dapp-dev-guide/deploying-contracts.html) and [Contracts on the Blockchain](https://docs.casperlabs.io/en/latest/dapp-dev-guide/calling-contracts.html).

After the contract is deployed, the contract owner account will be assigned as the first admin and will have the following named keys added:

Named key | Description
--------- | ------------
```account-info-latest-version-contract``` | The hash of the latest version of the contract
```account-info-latest-version-contract-hash``` | A URef to the value that stores the hash of the latest version of the contract
```account-info-package``` | The contract package hash
```account-info-package-hash``` | A URef to the value that stores the contract package hash
```account-info-admins``` | Seed URef to the dictionary that stores contract admins
```account-info-urls``` | Seed URef to the dictionary that stores account information URLs

## Contract API

The contract has two sets of entry points:
- public entry points, which should be used by Casper account owners to provide information about themselves
- admin entry points, which should be used by the contract administrators

### Public entry points

#### set_url

Sets a domain URL under which the account information file should be stored for the contract caller. Note, that only the top level domain without the ```.well-known/casper/account-info.<NETWORK_NAME>.json``` part should be provided

Arguments:

Name | Type | Description
---- | ---- | -----------
```url``` | ```String``` | Top level domain URL under which the account information file is stored

#### get_url

Returns the top level domain URL under which the account information file is stored for the given public key

Arguments:

Name | Type | Description
---- | ---- | -----------
```account``` | ```AccountHash``` | The account hash of the account the account information URL is requested for

#### delete_url

Deletes the top level domain URL under which the account information standard file is stored for the contract caller

Arguments: this entry point has no arguments 

### Admin entry points

The entry points below are available only to the accounts defined as admins.

#### set_url_for_account

Sets a URL to the account information standard file for the provided account. 

Arguments:

Name | Type | Description
---- | ---- | -----------
```url``` | ```String``` | Top level domain URL under which the account information file is stored
```account``` | ```AccountHash``` | The account has of the account, the information standard file URL should be set for

#### delete_url_for_account

Deletes the account information standard file URL from the provided account.

Arguments: 

Name | Type | Description
---- | ---- | -----------
```account``` | ```AccountHash``` | The account has of the account, the information standard file URL should be deleted from

#### add_admin

Add another admin account. Fails if the account already added as an admin.

Arguments: 

Name | Type | Description
---- | ---- | -----------
```account``` | ```AccountHash``` | The account hash of the new admin account.

#### disable_admin

Disables existing admin account. Fails if the account is not an admin or if there is only one admin account left.

Arguments: 

Name | Type | Description
---- | ---- | -----------
```account``` | ```AccountHash``` | The account of the existing admin account, that should be disabled

#### set_cspr_to_burn

Sets amount of CSPR that should be burned during the ```set_url``` entry point execution, increasing the execution price

Arguments:

Name | Type | Description
---- | ---- | -----------
```cspr_to_burn``` | ```U32``` | The account CSPR that should be burned during the ```set_url``` entry point execution

## Development

### Setup

```bash
make prepare
```

### Build

```bash
make build-contract
```

The WASM file will be available in the target directory:
```bash
target/wasm32-unknown-unknown/release/account-info.wasm
```

### Test

```bash
make test
```
