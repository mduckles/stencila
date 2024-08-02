import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Instruction } from './instruction'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'

/**
 * Web component representing a Stencila Schema `InstructionBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction-block.md
 */
@customElement('stencila-instruction-block')
@withTwind()
export class InstructionBlock extends Instruction {
  override render() {
    const { borderColour } = nodeUi('InstructionBlock')

    return html`<stencila-ui-block-on-demand
      type="InstructionBlock"
      node-id=${this.id}
      depth=${this.depth}
      ancestors=${this.ancestors}
    >
      <span slot="header-right" class="flex">
        <stencila-ui-node-execution-commands
          node-id=${this.id}
          type="InstructionBlock"
        >
        </stencila-ui-node-execution-commands>
      </span>

      <div slot="body">
        <stencila-ui-node-execution-details
          type="InstructionBlock"
          mode=${this.executionMode}
          .tags=${this.executionTags}
          status=${this.executionStatus}
          required=${this.executionRequired}
          count=${this.executionCount}
          ended=${this.executionEnded}
          duration=${this.executionDuration}
        >
          <slot name="execution-dependencies"></slot>
          <slot name="execution-dependants"></slot>
        </stencila-ui-node-execution-details>

        <stencila-ui-node-execution-messages
          type="InstructionBlock"
          warning-count=${this.warningCount}
          error-count=${this.errorCount}
        >
          <slot name="execution-messages"></slot>
        </stencila-ui-node-execution-messages>

        ${this.renderProperties()}

        <slot name="model"></slot>

        <div class="border-t border-[${borderColour}]">
          <slot name="message"></slot>
        </div>
      </div>

      <div slot="content" class="w-full">
        <slot name="content"></slot>
        <slot name="suggestions"></slot>
      </div>
    </stencila-ui-block-on-demand>`
  }

  /**
   * Render a ribbon style container with properties of the instruction
   */
  private renderProperties() {
    const { borderColour, colour } = nodeUi('InstructionBlock')

    const styles = apply(
      'flex flex-row items-center',
      'px-3 py-1.5',
      `bg-[${colour}]`,
      'text-xs leading-tight font-sans',
      `border-t border-[${borderColour}]`
    )

    const inputStyles = apply([
      `border border-[${borderColour}] rounded-sm`,
      `outline-[${borderColour}]/50`,
      'text-sm text-gray-600',
      'ml-1 p-1',
    ])

    return html`
      <div class=${styles}>
        <span class="grow">
          <sl-tooltip content="Assistant assigned to perform the instruction">
            <sl-icon class="text-base" name="at"></sl-icon>
          </sl-tooltip>
          <input
            class="${inputStyles} w-[70%]"
            type="text"
            value=${this.assignee}
            readonly
            disabled
          />
        </span>

        <span>
          <sl-tooltip content="Number of suggestions to generate">
            <sl-icon class="text-base" name="hash"></sl-icon>
          </sl-tooltip>
          <input
            class="${inputStyles}"
            type="number"
            min="1"
            max="10"
            value=${this.replicates ?? 1}
            readonly
            disabled
          />
        </span>
      </div>
    `
  }
}
