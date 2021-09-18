import * as near from '@4ire-labs/near-sdk'
import * as util from '../../util'
import fs from 'fs'
import path from 'path'

util.config('testnet');
const environment = near.environment()
const rootPath = path.dirname(path.dirname(__dirname))

function id(name: string): string {
    const accountId = process.env[`NEAR_${name.toUpperCase()}_ID`] || ''
    if (accountId === '') {
        throw new Error(`Not set NEAR_${name.toUpperCase()}_ID`)
    }
    return accountId
}

async function deploy(name: string, init?: near.ChangeMethod) {
    const accountId = id(name);
    console.log(`[${environment.networkId}] DEPLOY "${name}" IN PROGRESS`)
    const sender = await near.readUnencryptedFileSystemKeyStore(accountId);
    if (!(await near.isExistAccount(sender))) {
        throw new Error(`Account ${sender.accountId} not exist`)
    }
    const contractCode = fs.readFileSync(`${rootPath}/target/wasm32-unknown-unknown/release/${name}.wasm`)
    console.log('sender:', {
        networkId: sender.networkId,
        accountId: sender.accountId,
        publicKey: sender.keyPair.publicKey.toString(),
    })
    const props = <near.DeployProps>{sender}
    if ((await near.stateAccount(sender)).code_hash === '11111111111111111111111111111111') {
        props.init = init
    }
    const contractResult = await near.deployContract(sender, contractCode, props)
    console.log(`[${environment.networkId}] Deployed contract:`, {
        accountId: contractResult.account.accountId,
        transactionId: contractResult.outcome.transactionId,
    })
    console.log(`[${environment.networkId}] DEPLOY "${name}" DONE`)
    return sender
}

async function main() {
    const refFarmingAccount = await deploy('ref_farming_strategy', <near.ChangeMethod>{
        methodName: 'new',
        args: { 
            executor: 'strategy.testnet', 
            ref_exchange_account: 'ref-exchange-strategy.testnet', 
            ref_farming_account: 'ref-farming-strategy.testnet' 
        }
    })
    // const depositum = await near.Contract.connect(near.Contract, refFarmingAccount.accountId, refFarmingAccount)
    // let callResult = await depositum.callRaw<void>({
    //     methodName: 'accounts_list',
    //     args: { }
    // })
    // console.log(`[${refFarmingAccount.networkId}] accounts_list: ${callResult.transactionId}`)
}

main().catch((error) => {
    console.error(error)
    process.exit(1)
})


/*
// create account 
const accountId = 'farm-rant.testnet';
    const sender1 = near.parseAccountNetwork(near.accountIdBySlug('rant'));
    const account = near.custodianAccount(accountId, sender1);
    const tx = await near.createAccount(sender1,account, '10');
    console.log('create acc tx', tx);
    return;
*/