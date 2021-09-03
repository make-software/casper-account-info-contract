#!/bin/bash

print_usage () {
  echo "USAGE:"
  echo "  is-admin.sh [ARGUMENTS]"
  echo
  echo "ARGUMENTS:"
  echo "  --node-address   Casper node to run RPC requests against (default: 127.0.0.1)"
  echo "  --contract-hash  Account info contract hash without the 'hash--' prefix (default: 2f36a35edcbaabe17aba805e3fae42699a2bb80c2e0c15189756fdc4895356f8, account info contract hash on the Testnet)"
  echo "  --public-key     Account public key in the hex format"
  echo
  echo "EXAMPLE:"
  echo "  is-admin.sh --public-key=0106ca7c39cd272dbf21a86eeb3b36b7c26e2e9b94af64292419f7862936bca2ca"
  echo
  echo "DEPENDENCIES:"
  echo "  casper-client    To make RPC requests to the network"
  echo "  jq               To parse RPC responses"
}

ensure_has_installed () {
  HAS_INSTALLED=$(which "$1")
  if [ "$HAS_INSTALLED" = "" ]; then
    echo "Please install $1"
    exit 1
  fi
}

ensure_has_installed "casper-client"
ensure_has_installed "jq"

while [ $# -gt 0 ]; do
  case "$1" in
    --node-address=*)
      NODE_ADDRESS="${1#*=}"
      ;;
    --contract-hash=*)
      CONTRACT_HASH="${1#*=}"
      ;;
    --public-key=*)
      PUBLIC_KEY="${1#*=}"
      ;;
    *)
      print_usage; exit 1
      ;;
  esac
  shift
done

if [ -z ${NODE_ADDRESS+x} ]; then NODE_ADDRESS=127.0.0.1; fi
if [ -z ${CONTRACT_HASH+x} ]; then CONTRACT_HASH=2f36a35edcbaabe17aba805e3fae42699a2bb80c2e0c15189756fdc4895356f8; fi
if [ -z ${PUBLIC_KEY+x} ]; then print_usage; exit 1; fi

STATE_ROOT_HASH=$(casper-client get-state-root-hash --node-address http://$NODE_ADDRESS:7777 | jq -r '.result | .state_root_hash')

ACCOUNT_INFO_ADMINS_DICT_UREF=$(casper-client query-state \
  --node-address http://$NODE_ADDRESS:7777 \
  --state-root-hash "$STATE_ROOT_HASH" \
  --key "hash-$CONTRACT_HASH" \
| jq -rc '.result | .stored_value | .Contract | .named_keys | map(select(.name | contains("account-info-admins"))) | .[] .key')

ACCOUNT_HASH=$(casper-client account-address -p $PUBLIC_KEY | sed -r 's/account-hash-//g')

IS_ADMIN=$(casper-client get-dictionary-item \
  --node-address http://$NODE_ADDRESS:7777 \
  --state-root-hash "$STATE_ROOT_HASH" \
  --seed-uref  "$ACCOUNT_INFO_ADMINS_DICT_UREF" \
  --dictionary-item-key "$ACCOUNT_HASH" \
| jq -r '.result | .stored_value | .CLValue | .parsed')

CHAIN_NAME=$(curl -s http://$NODE_ADDRESS:8888/status | jq -r '.chainspec_name')

if [ "$IS_ADMIN" = "true" ]; then
  echo "$PUBLIC_KEY is an admin"
else
  echo "$PUBLIC_KEY is not an admin"
fi
