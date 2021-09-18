import {Contract,} from 'near-api-js'

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
  async coin_list() {
    return (await this.contract.coin_list()).reduce((acc, it) => {
      acc[it] = {}
      // TODO add metadata
      return acc
    },{})
  }

  /**
   * @param account_id
   * @returns {Object}
   */
  async balance_of(account_id) {
    return (await this.contract.balance_of(account_id)).reduce((acc, it) => {
      acc[it[0]].amount = it[1]
      return acc
    }, (await this.coin_list()))
  }
}
