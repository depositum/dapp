import {Injectable} from '@angular/core'
import {
  config,
  configList,
  NetworkConfig,
} from '../config'
import {
  connect,
  keyStores,
  WalletConnection,
  ConnectConfig,
  Account,
} from 'near-api-js/src'
import {
  formatNearAmount,
  parseNearAmount,
} from 'near-api-js/lib/utils/format'
import {Depositum} from '../../contract/depositum/depositum'
// import {ConnectConfig} from "near-api-js/src/connect";

const toNear = (balance: string) => Math.floor(parseFloat(formatNearAmount(balance)) * 100) / 100
const envDefault = 'testnet'

@Injectable({
  providedIn: 'root',
})
export class WalletService {
  connection?: WalletConnection
  contractName?: string
  contract?: Depositum
  accountId?: string
  config?: NetworkConfig
  configList: NetworkConfig[] = []
  // Depositum
  coinList: string[] = []

  constructor() {
    this.configList = configList()
    this.initContract(envDefault).catch(reason => {
      throw new Error(reason.message)
    })
  }

  async initContract(env: string): Promise<void> {
    this.config = config(env) // TODO implement detect env
    // FIXME yarn build_web [error] SyntaxError: build/web/main-es2015.XXX.js: Deleting local variable in strict mode. (1:259472)
    //const cfg = <ConnectConfig>this.config
    //cfg.keyStore = new keyStores.BrowserLocalStorageKeyStore()
    //const near = await connect(cfg)
    const near = await connect(Object.assign({
      deps: {
        keyStore: new keyStores.BrowserLocalStorageKeyStore(),
      },
    }, this.config))
    this.connection = new WalletConnection(near, this.config.networkId)
    this.accountId = this.connection.getAccountId()
    this.contractName = this.config.contractName
    this.contract = new Depositum(this.connection.account(), this.contractName)
    await this.update()
  }

  async update(): Promise<void> {
    await this.updateCoinList()
  }

  async updateCoinList(): Promise<void> {
    this.coinList = await this.contract?.coin_list() || []
  }

  signIn(): void {
    this.connection?.requestSignIn(this.config?.contractName)
  }

  isAuthenticated(): boolean {
    return !!this.accountId
  }

  signOut(): void {
    this.connection?.signOut()
    this.accountId = ''
  }
}
