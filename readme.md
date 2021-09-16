<div align="center">
  <h1><code>Novum Depositum at MetaBUIDL 2021</code></h1>
  <p>
    <strong>WiP TBD</strong>
  </p>
</div>

## Local config

```shell
yarn install
yarn setup
npm install --global near-cli
NEAR_ENV=local near send alice.local bob.local 1.42
NEAR_ENV=local near state depositum.local
NEAR_ENV=local near --accountId bob.local call ref-finance.local storage_deposit '{"account_id": "bob.local", "registration_only": false}' --accountId bob.local --amount 0.1
NEAR_ENV=local near --accountId bob.local call  ref-finance.local storage_deposit '{"account_id": "alice.local", "registration_only": false}' --accountId bob.local --amount 0.1
NEAR_ENV=local near --accountId bob.local call usdt.local ft_transfer_call "{\"receiver_id\": \"ref-finance.local\", \"amount\": \"1000000000000\", \"msg\": \"\"}" --accountId bob.local --amount 0.000000000000000000000001
```

## Util

```shell
NEAR_ENV=testnet bash util/simple_token_deploy.sh
NEAR_ENV=testnet bash util/simple_token_delete.sh
```
## Develop

```shell
make fix 
make qa
make build
make clean
```

### QA

```shell
make qa
```

## Deploy web
```shell
yarn deploy_web
```

## Deploy contract

### stage

* `cp .env.test .env` for `testnet`
* `cp .env.beta .env` for `betanet`
* `cp .env.local .env` for `local` and run `yarn setup`

```shell
make rebuild
```
> ⚠️The operations not idempotent at yet
```shell
yarn setup_state
yarn deploy_contract
```

### mainnet

TBD


## Usage

```shell
source <(< .env xargs -n1 echo "export $1")
near view ${NEAR_DEPOSITUM_ID} coin_list
near view ${NEAR_DEPOSITUM_ID} strategy_list
near view ${NEAR_DEPOSITUM_ID} balance_of '{"account_id": "<account_id>"}'
```
