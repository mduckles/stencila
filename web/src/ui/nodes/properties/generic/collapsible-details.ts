import { NodeType } from '@stencila/types'
import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../../twind'

import './collapsible'

/**
 * UI Node Collapsible Details
 *
 * A component to render inside a node-card, which allows its content to be
 * collapsed & hidden. This includes the necessary styling of the header &
 * the shell of the body.
 */
@customElement('stencila-ui-node-collapsible-details')
@withTwind()
export class UINodeCollapsibleDetails extends LitElement {
  @property()
  type: NodeType

  @property({ attribute: 'icon-name' })
  iconName: string

  @property({ type: Boolean })
  collapsed: boolean = false

  @property({ attribute: 'wrapper-css' })
  wrapperCss: string | undefined = 'border-t border-black/30'

  static override styles = css`
    ::slotted(*) {
      display: flex;
      flex-direction: column;
      row-gap: 0.75rem; // gap-y-3
    }
  `

  override render() {
    return html`
      <stencila-ui-node-collapsible-property
        type=${this.type}
        icon-name=${this.iconName}
        .collapsed=${this.collapsed}
        wrapper-css=${this.wrapperCss}
      >
        <div slot="title">${this.title}</div>
        <div class="px-4 py-3" slot="content">
          <slot></slot>
        </div>
      </stencila-ui-node-collapsible-property>
    `
  }
}
