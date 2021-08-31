import * as near from '@4ire-labs/near-sdk'
import * as util from '.'

util.config()

async function main() {
    console.log('[SETUP] IN PROGRESS')
    const sender = near.parseAccountNetwork(near.accountIdBySlug('alice'))
    const finance = await near.Contract.connect(near.Contract, near.accountIdBySlug('ref-finance'), sender)
    // near call $CONTRACT_ID add_simple_pool "{\"tokens\": [\"$TOKEN1\", \"$TOKEN2\"], \"fee\": 25}"
    // --accountId $USER_ID --amount 0.1
    const add_simple_pool_out = await finance.callRaw({
        methodName: 'add_simple_pool',
        args: {
            tokens: [
                near.accountIdBySlug('usdt'),
                near.accountIdBySlug('wnear'),
            ],
            fee: 25,
        },
        attachedDeposit: '0.1',
    })
    console.log(add_simple_pool_out.transactionId)
}

main().catch(console.error)
