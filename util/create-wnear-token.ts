import * as near from '@4ire-labs/near-sdk'
import * as util from '.'
const NETWORK = process.env.NEAR_ENV;
util.config(NETWORK);
const ACCOUNT: string | undefined = process.env.MAIN_ACCOUNT || '';

const WNEAR_TOKEN_ACC_NAME = near.accountIdBySlug(`wrap_near-${ACCOUNT}`);

const sender = near.parseAccountNetwork(near.accountIdBySlug(ACCOUNT));

async function createWrapperNear() {
    console.log('[SETUP] CREATE TOKEN CONTRACTS');

    const account = near.custodianAccount(WNEAR_TOKEN_ACC_NAME, sender);
    if (!(await near.isExistAccount(account))) {
        const tx = await near.createAccount(sender, account, '10')
        console.log('[SETUP] Created account:', { accountId: account.accountId, transactionId: tx.transactionId, });
    }

    const token = await near.fetchContract('wrap.near', 'mainnet');
    // create wNEAR contract
    const wnearAcc = near.custodianAccount(WNEAR_TOKEN_ACC_NAME);
    const codeHash = (await (await near.accountConnect(wnearAcc)).state()).code_hash;
    if (codeHash == '11111111111111111111111111111111') {
        try {
            const result = await near.deployContract(wnearAcc, token, {
                init: { methodName: 'new', args: {} },
                amount: '10', sender: wnearAcc
            });

            console.log('[SETUP] contract deployed:', { transactionId: result.outcome.transactionId, });
        } catch (err) { }
    }
}

(async () => {
    await createWrapperNear();
})();