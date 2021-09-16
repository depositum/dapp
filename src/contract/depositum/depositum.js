import {
  Contract,
} from 'near-api-js'

export class Depositum {
  /**
   * @type {Contract}
   */
  contract

  constructor(account, contractId) {
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

  /**
   * @returns {Promise<string[]>}
   */
  coin_list() {
    return this.contract.coin_list()
  }

  /**
   * @param account_id
   * @returns {Promise<Array<Array<[string, string]>>>}
   */
  balance_of(account_id) {
    return this.contract.balance_of(account_id)
  }
}
