import '@shoelace-style/shoelace/dist/components/button/button.js'
import '@shoelace-style/shoelace/dist/components/popup/popup.js'

import { css } from '@twind/core'
import { html } from 'lit'
import { customElement, state } from 'lit/decorators'

import { insertClones } from '../../clients/commands'
import { Entity } from '../../nodes/entity'
import { withTwind } from '../../twind'

import { UIBaseClass } from './mixins/ui-base-class'

@customElement('stencila-ui-nodes-selected')
@withTwind()
export class UINodesSelected extends UIBaseClass {
  /**
   * The selected nodes
   *
   * A `@state` so that when updated (due to selection change)
   * the popup changes.
   */
  @state()
  private selectedNodes: string[][] = []

  /**
   * The position of the anchor for the popup
   *
   * Does not need to be a `@state` because only ever updated
   * when `selectedNodes` is updated.
   */
  private anchorPosition = { x: 0, y: 0 }

  @state()
  active: boolean = false

  /**
   * Checks the element is the right container for the selection functionality,
   *
   * Should be a div with a slot="content" attibute,
   * Must have a parent element like so: `stencila-chat-message[message-role="Model"]`.
   */
  private isTargetContainer(element: Element | null) {
    return (
      element?.tagName === 'DIV' &&
      element.getAttribute('slot') === 'content' &&
      element.parentElement?.tagName.toLowerCase() ===
        'stencila-chat-message' &&
      element.parentElement?.getAttribute('message-role') === 'Model'
    )
  }

  override connectedCallback() {
    super.connectedCallback()
    document.addEventListener(
      'selectionchange',
      this.handleSelectionChange.bind(this)
    )
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    document.removeEventListener(
      'selectionchange',
      this.handleSelectionChange.bind(this)
    )
  }

  /**
   * Handle a change in the selection
   */
  handleSelectionChange() {
    this.active = false
    const selection = window.getSelection()
    if (!selection.rangeCount) {
      this.selectedNodes = []
      return
    }

    // Get the common ancestor of the selected range
    const range = selection.getRangeAt(0)

    let container =
      range.commonAncestorContainer.nodeType == Node.TEXT_NODE
        ? range.commonAncestorContainer.parentElement
        : (range.commonAncestorContainer as Element)

    // Walk up out of the ancestor element until we get
    // to a node type that has block content
    while (container && !this.isTargetContainer(container)) {
      container = container.parentElement
    }

    if (!container) {
      this.selectedNodes = []
      return
    }

    // Get selected nodes from direct children
    const selectedNodes = []
    const children = container.children
    for (const child of children) {
      if (range.intersectsNode(child) && child instanceof Entity && child.id) {
        const type = child.nodeName
        selectedNodes.push([type, child.id])
      }
    }

    if (selectedNodes.length > 0) {
      // Position anchor element near the selection
      const rect = range.getBoundingClientRect()
      this.anchorPosition = {
        x: rect.left,
        y: rect.bottom,
      }
      this.active = true
    }

    this.selectedNodes = selectedNodes
  }

  async insertIds() {
    // Send command to insert nodes into document
    const ids = this.selectedNodes.map(([_, id]) => id)
    this.dispatchEvent(insertClones(ids))

    // Clear selection after successful insertion
    window.getSelection().removeAllRanges()

    // Clear the selected nodes so popup is hidden
    this.selectedNodes = []
  }

  override render() {
    const tagStyles = css`
      &::part(base) {
        display: flex;
        justify-content: space-between;
      }
    `

    return html`
      <div
        id="stencila-nodes-selected-anchor"
        style="
          position:fixed;
          left:${this.anchorPosition.x}px;
          top:${this.anchorPosition.y}px"
      ></div>

      <sl-popup
        anchor="stencila-nodes-selected-anchor"
        placement="bottom-start"
        distance="10"
        ?active=${this.active}
      >
        <div
          class="p-3 bg-brand-blue text-white font-sans text-sm border border-white rounded"
        >
          <div class="flex justify-center mb-2">
            <button class="flex flex-col items-center" @click=${this.insertIds}>
              <stencila-ui-icon
                name="boxArrowInLeft"
                class="text-lg"
              ></stencila-ui-icon>
              Insert
            </button>
          </div>
          <div class="flex flex-col gap-y-2">
            ${this.selectedNodes.map(
              ([_type, nodeId]) => html`
                <sl-tag
                  size="small"
                  class=${tagStyles}
                  removable
                  @sl-remove=${() => {
                    this.selectedNodes = this.selectedNodes.filter(
                      ([_, id]) => id !== nodeId
                    )
                  }}
                >
                  ${nodeId}
                </sl-tag>
              `
            )}
          </div>
        </div>
      </sl-popup>
    `
  }
}
