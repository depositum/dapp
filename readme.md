<div align="center">
  <h1><code>Novum Depositum at MetaBUIDL 2021</code></h1>
  <p>
    <strong>WiP TBD</strong>
  </p>
</div>

## Local environment

```shell
yarn install
yarn setup
source util/near-shortcut.sh
near-local send alice.local bob.local 1.42
NEAR_ENV=local npx near --keyPath <(local-key bob.local) call  ref-finance.local storage_deposit '{"account_id": "bob.local", "registration_only": false}' --accountId bob.local --amount 0.1
NEAR_ENV=local npx near --keyPath <(local-key bob.local) call  ref-finance.local storage_deposit '{"account_id": "alice.local", "registration_only": false}' --accountId bob.local --amount 0.1


NEAR_ENV=local npx near --keyPath <(local-key bob.local) call usdt.local ft_transfer_call "{\"receiver_id\": \"ref-finance.local\", \"amount\": \"1000000000000\", \"msg\": \"\"}" --accountId bob.local --amount 0.000000000000000000000001
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

## Deploy

```shell
make rebuild
```

### local

```shell
yarn setup
source util/near-shortcut.sh
near-local --masterAccount local create-account coin.local
near-local --masterAccount coin.local deploy --accountId coin.local --initFunction new --initArgs '{}' --wasmFile build/simple_token.wasm
near-local --masterAccount local create-account depositum.local
```
`.env`
```ini
NEAR_ENV=local
NEAR_DEPOSITUM_ID=depositum.local
NEAR_COIN_ID=coin.local
```

### testnet

```shell
near dev-deploy build/depositum.wasm && near delete $(cat neardev/dev-account) depositum.testnet && rm -rf neardev
near --masterAccount depositum.testnet create-account alpha.depositum.testnet --initialBalance 50
near --masterAccount depositum.testnet create-account sandbox.depositum.testnet --initialBalance 50
near dev-deploy build/depositum.wasm && near delete $(cat neardev/dev-account) sandbox.depositum.testnet && rm -rf neardev
near --masterAccount sandbox.depositum.testnet create-account coin.sandbox.depositum.testnet --initialBalance 50

near --masterAccount coin.sandbox.depositum.testnet deploy --accountId coin.sandbox.depositum.testnet --initFunction new --initArgs '{}' --wasmFile build/simple_token.wasm
```

`.env`
```ini
NEAR_ENV=testnet
NEAR_DEPOSITUM_ID=alpha.depositum.testnet
NEAR_COIN_ID=coin.sandbox.depositum.testnet
```

### mainnet

TBD

### deploy

```shell
yarn deploy
```

## Usage

TBD
