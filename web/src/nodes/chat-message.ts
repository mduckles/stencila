import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { html, PropertyValues } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { booleanConverter } from '../utilities/booleanConverter'
import { closestGlobally } from '../utilities/closestGlobally'

import { Executable } from './executable'

import '../ui/nodes/chat-message-inputs'

/**
 * Web component representing a Stencila `ChatMessage` node
 *
 * Renders differently depending upon whether the messages is a system
 * message (i.e. a system prompt), a user message (i.e. an instruction
 * from the user), or a model message (i.e a response from a model).
 *
 * User messages are only editable, and only have a toolbar, if they
 * have not yet been "executed" successfully.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/chat-message.md
 */
@customElement('stencila-chat-message')
@withTwind()
export class ChatMessage extends Executable {
  @property({ attribute: 'message-role' })
  messageRole: 'System' | 'User' | 'Model'

  @property({
    attribute: 'is-selected',
    converter: booleanConverter,
  })
  isSelected?: boolean = false

  /**
   * Should a node card, possibly within a chat message, be expanded?
   */
  public static shouldExpand(card: HTMLElement, nodeType: NodeType): boolean {
    const types: NodeType[] = [
      'CodeBlock',
      'CodeChunk',
      'Datatable',
      'Figure',
      'ForBlock',
      'IfBlock',
      'IncludeBlock',
      'InstructionBlock',
      'MathBlock',
      'RawBlock',
      'StyledBlock',
      'Table',
    ]

    return (
      types.includes(nodeType) &&
      closestGlobally(card, 'stencila-chat-message[message-role="Model"]') !==
        null
    )
  }

  protected override firstUpdated(changedProperties: PropertyValues): void {
    super.firstUpdated(changedProperties)
    // set first message in group as selected by default
    if (this.isWithin('ChatMessageGroup')) {
      this.isSelected = this.parentNode.children[0].isSameNode(this)
    }
  }

  override render() {
    // These styles are applied here, rather than any container in
    // a chat because in a chat message group the messages within
    // each group need to be limited in width
    const style = apply('min-w-[45ch] max-w-prose mx-auto')

    switch (this.messageRole) {
      case 'System':
        return this.renderSystemMessage(style)
      case 'User':
        return this.renderUserMessage(style)
      case 'Model':
        return this.renderModelMessage(style)
    }
  }

  private renderSystemMessage(style: string) {
    return html`
      <div class="${style} my-3 p-3 bg-indigo-100 rounded">
        <slot name="content"></slot>
      </div>
    `
  }

  private renderUserMessage(style: string) {
    return html`
      <div class="${style} flex justify-end">
        <div class="my-3 p-3 bg-blue-50 rounded w-content">
          <slot name="content"></slot>
          <slot name="files"></slot>
        </div>
      </div>
    `
  }

  private renderModelMessage(style: string) {
    const inGroup = this.isWithin('ChatMessageGroup')

    if (!inGroup) {
      return html`
        <div class="${style} my-3">
          <slot name="author" class="text-blue-900"></slot>
          ${this.executionStatus === 'Running'
            ? this.renderRunningIndicator()
            : html`<slot name="content"></slot>`}
        </div>
      `
    }

    return this.isSelected
      ? html`
          <div class="${style} my-3">
            ${this.executionStatus === 'Running'
              ? this.renderRunningIndicator()
              : html`<slot name="content"></slot>`}
          </div>
        `
      : html``
  }

  private renderRunningIndicator() {
    const dotClasses = apply('h-2 w-2 bg-gray-500 rounded-full animate-bounce')

    return html`
      <div
        class="flex justify-center items-center gap-x-1 mt-3 p-5 rounded bg-gray-100 w-full"
      >
        <div class=${dotClasses} style="animation-delay: 0ms"></div>
        <div class=${dotClasses} style="animation-delay: 250ms"></div>
        <div class=${dotClasses} style="animation-delay: 500ms"></div>
      </div>
    `
  }
}
