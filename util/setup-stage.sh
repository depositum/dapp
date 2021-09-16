#!/usr/bin/env bash

set -o errexit

ROOT_PATH=$(cd "$( dirname "${BASH_SOURCE[0]}" )" && cd ../ && pwd)

# shellcheck disable=SC1090
source <(< "${ROOT_PATH}/.env" xargs -n1 echo "export $1")

printenv | grep NEAR | sort

if [ "${NEAR_ENV}" == 'local' ]; then # TODO remo after release https://github.com/near/near-cli/pull/838
  near --helperUrl http://localhost:3000 dev-deploy <(printf "")
else
  near dev-deploy <(printf "")
fi
near delete "$(cat neardev/dev-account)" "${NEAR_MASTER_ID}" && rm -rf neardev
near --masterAccount "${NEAR_MASTER_ID}" create-account "${NEAR_DEPOSITUM_ID}" --initialBalance 10
near --masterAccount "${NEAR_MASTER_ID}" create-account "${NEAR_SANDBOX_ID}" --initialBalance 10
near --masterAccount "${NEAR_SANDBOX_ID}" create-account "${NEAR_COIN_ID}" --initialBalance 10
near --masterAccount "${NEAR_COIN_ID}" deploy --accountId "${NEAR_COIN_ID}" --initFunction new --initArgs '{}' --wasmFile build/simple_token.wasm
