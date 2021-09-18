import {
  Contract,
  Account,
} from 'near-api-js/src'
interface DepositumInterface {
  coin_list(): Promise<string[]>
  balance_of(account_id: Account): Promise<Array<Array<[string, string]>>>
}
export class Depositum {
  private contract: DepositumInterface
  constructor(account: Account, contractId: string) {
    // @ts-ignore
    this.contract = new Contract(account, contractId, {
      viewMethods: [
        'coin_list',
        'strategy_list',
        'balance_of',
      ],
      changeMethods: [
        'coin_enable',
        'coin_disable',
      ],
    })
  }
  coin_list(): Promise<string[]> {
    return this.contract.coin_list()
  }
  balance_of(account_id: Account): Promise<Array<Array<[string, string]>>> {
    return this.contract.balance_of(account_id)
  }
}
