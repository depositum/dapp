import {Component} from '@angular/core'
import pkg from '../../../package.json'

@Component({
  selector: 'app-root',
  template: `
    <app-panel
      [title]="title"
      [description]="description"></app-panel>
    <router-outlet></router-outlet>
  `,
  styles: [],
})
export class AppComponent {
  title = pkg.name.toUpperCase()
  description = pkg.description
}
