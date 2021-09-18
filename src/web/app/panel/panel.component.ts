import {
  Component,
  Input,
} from '@angular/core'
import {WalletService} from '../wallet.service'

@Component({
  selector: 'app-panel',
  template: `
<!--    <div class="grid-container" *ngIf="!wallet.isAuthenticated()">-->
    <div class="grid-container">
      <div class="grid-x grid-padding-x" *ngIf="title || description">
        <div class="large-12 cell align-center">
          <h1 class="title" *ngIf="title">{{title}}</h1>
          <span class="description" *ngIf="description" style="display: block">{{description}}</span>
          <img width="255" [alt]="title" src="../../asset/logo.svg">
        </div>
      </div>
      <div class="grid-x grid-padding-x">
        <div class="large-12 cell align-center">
<!--          <button class="button" (click)="wallet.signIn()">Connect</button>-->
        </div>
      </div>
    </div>
  `,
  styles: [`
    :host {
      text-align: center;
      display: block;
    }
    .title {
      font-size: 2rem;
      font-weight: bold;
    }
    .description {
      font-size: 1.5rem;
    }
  `],
})
export class PanelComponent {
  @Input() title?: string;
  @Input() description?: string;

  constructor(private wallet: WalletService) {}
}
