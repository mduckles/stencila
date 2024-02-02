import { apply } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'
import { DocumentView, NodeType } from '../../types'
import { getNodeColour, getNodeIcon } from '../../ui/nodes/nodeMapping'

/**
 * A component for displaying information about a `Block` node type (e.g. a `Heading` or `Table`)
 */
@customElement('stencila-block-infobox')
@withTwind()
export class BlockInfobox extends LitElement {
  @property()
  icon: string = ''

  @property()
  colour: string = ''

  @property()
  currentNode: NodeType

  @property()
  view: DocumentView

  @property()
  override title: string = ''

  override render() {
    const colour = getNodeColour(this.currentNode)
    const icon = getNodeIcon(this.currentNode)
    const styles = apply([
      'w-full',
      'p-4',
      'bg-white',
      `border border-[${this.colour}] rounded`,
    ])

    // TODO: design this
    return html`
      <div class=${styles} style="background-color: ${colour};">
        <span>
          <sl-icon name=${icon} library="stencila"></sl-icon>
          ${this.title}
        </span>
        <slot name="authors"></slot>
        <slot name="items"></slot>
      </div>
    `
  }
}
