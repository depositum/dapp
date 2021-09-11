import * as near from '@4ire-labs/near-sdk'
import * as util from '../../util'
import fs from 'fs'
import path from 'path'
import {DeployProps} from "@4ire-labs/near-sdk/contract";

util.config('')
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
    const accountId = id(name)
    console.log(`[${environment.networkId}] DEPLOY "${name}" IN PROGRESS`)
    const sender = await near.readUnencryptedFileSystemKeyStore(accountId)
    if (!(await near.isExistAccount(sender))) {
        throw new Error(`Account ${sender.accountId} not exist`)
    }
    const contractCode = fs.readFileSync(`${rootPath}/build/${name}.wasm`)
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
    const depositumAccount = await deploy('depositum', <near.ChangeMethod>{
        methodName: 'new',
    })
    const depositum = await near.Contract.connect(near.Contract, depositumAccount.accountId, depositumAccount)
    let callResult = await depositum.callRaw<void>({
        methodName: 'coin_enable',
        args: { coin: id('coin') }
    })
    console.log(`[${depositumAccount.networkId}] coin_enable: ${callResult.transactionId}`)
}

main().catch((error) => {
    console.error(error)
    process.exit(1)
})
