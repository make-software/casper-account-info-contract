#!/bin/bash

print_usage () {
  echo "USAGE:"
  echo "  get-cspr-to-burl-value.sh [ARGUMENTS]"
  echo
  echo "ARGUMENTS:"
  echo "  --node-address   Casper node to run RPC requests against (default: 127.0.0.1)"
  echo "  --contract-hash  Account info contract hash without the 'hash--' prefix (default: 2f36a35edcbaabe17aba805e3fae42699a2bb80c2e0c15189756fdc4895356f8, account info contract hash on the Testnet)"
  echo
  echo "EXAMPLE:"
  echo "  ./get-cspr-to-burl-value.sh"
  echo
  echo "DEPENDENCIES:"
  echo "  casper-client    To make RPC requests to the network"
  echo "  jq               To parse RPC responses"
  echo "  curl             To fetch account information"
  echo "  sed              To manipulate strings"
}

ensure_has_installed () {
  HAS_INSTALLED=$(which "$1")
  if [ "$HAS_INSTALLED" = "" ]; then
    echo "Please install $1"
    exit 1
  fi
}

ensure_has_installed "casper-client"
ensure_has_installed "curl"
ensure_has_installed "jq"
ensure_has_installed "sed"

while [ $# -gt 0 ]; do
  case "$1" in
    --node-address=*)
      NODE_ADDRESS="${1#*=}"
      ;;
    --contract-hash=*)
      CONTRACT_HASH="${1#*=}"
      ;;
    *)
      print_usage; exit 1
      ;;
  esac
  shift
done

if [ -z ${NODE_ADDRESS+x} ]; then NODE_ADDRESS=127.0.0.1; fi
if [ -z ${CONTRACT_HASH+x} ]; then CONTRACT_HASH=2f36a35edcbaabe17aba805e3fae42699a2bb80c2e0c15189756fdc4895356f8; fi

STATE_ROOT_HASH=$(casper-client get-state-root-hash --node-address http://$NODE_ADDRESS:7777 | jq -r '.result | .state_root_hash')

CSPR_TO_BURN_VALUE_UREF=$(casper-client query-state \
  --node-address http://$NODE_ADDRESS:7777 \
  --state-root-hash "$STATE_ROOT_HASH" \
  --key "hash-$CONTRACT_HASH" \
| jq -rc '.result | .stored_value | .Contract | .named_keys | map(select(.name | contains("cspr_to_burn"))) | .[] .key')

STATE_ROOT_HASH=$(casper-client get-state-root-hash --node-address http://$NODE_ADDRESS:7777 | jq -r '.result | .state_root_hash')

CSPR_TO_BURN_VALUE=$(casper-client query-state --node-address http://$NODE_ADDRESS:7777 --key "$CSPR_TO_BURN_VALUE_UREF" --state-root-hash "$STATE_ROOT_HASH" | jq -r '.result | .stored_value | .CLValue | .parsed')

echo "CSPR to burn value is $CSPR_TO_BURN_VALUE"
