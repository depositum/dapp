import {
  Component,
} from '@angular/core';
import {
  WalletService,
} from "./service/wallet.service";


@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.sass'],
})
export class AppComponent {
  title = 'Depositum'
  showHelp = false

  constructor(
    public wallet: WalletService,
  ) {
  }

  toggleHelp(): void {
    this.showHelp = !this.showHelp
  }

  titleFull(): string {
    return `${this.title} (${this.wallet.contractName})`
  }
}
