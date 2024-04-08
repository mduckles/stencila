import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../ui/nodes/card'

import { withTwind } from '../twind'

import { Entity } from './entity'

import './array-item'
import '../ui/nodes/node-card/in-flow/block'

/**
 * Web component representing a Stencila Schema `Array` node
 *
 * Note that this extends `Entity`, despite not doing so in Stencila Schema, to
 * make use of the various `render*View()` methods.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/array.md
 */
@customElement('stencila-array')
@withTwind()
export class Array extends Entity {
  /**
   * In static view just render the value
   */
  override renderStaticView() {
    return html`<slot></slot>`
  }

  /**
   * In dynamic view, in addition to the value, render a node card.
   */
  override renderDynamicView() {
    return html`
      <stencila-ui-block-in-flow type="Array" view="dynamic">
        <div slot="body" class="p-2">
          <slot></slot>
        </div>
      </stencila-ui-block-in-flow>
    `
  }

  /**
   * In source view, render the same as dynamic view, including the
   * value since that won't be a present in the source usually (given this
   * node type is normally only present in `CodeChunk.outputs` and `CodeExpression.output`).
   */
  override renderSourceView() {
    return html`
      <stencila-ui-block-in-flow type="Array" view="source">
        <div slot="body" class="p-2">
          <slot></slot>
        </div>
      </stencila-ui-block-in-flow>
    `
  }
}
