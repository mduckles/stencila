import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'
import StencilaDivision from '../../../nodes/division'
import { StencilaStyledView, styledAttrs } from './styled'

export function division(): NodeSpec {
  return {
    group: 'BlockContent',
    // Use +, rather than *, here so that if the `For` has no content
    // that at least a empty placeholder paragraph will be available for user to edit
    content: 'BlockContent+',
    // Necessary for copy/paste-ability of whole node, not just its content
    defining: true,
    attrs: styledAttrs,
    parseDOM,
    toDOM,
  }
}

export class StencilaDivisionView extends StencilaStyledView<StencilaDivision> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-division',
    getAttrs,
    contentElement: '[slot=content]',
  },
]

function getAttrs(node: StencilaDivision): Attrs {
  return {
    id: node.id,
    programmingLanguage: node.getAttribute('programming-language'),
    guessLanguage: node.getAttribute('guess-language'),
    code: node.querySelector('[slot=code]')?.innerHTML ?? '',
    css: node.querySelector('[slot=css]')?.innerHTML,
    errors: node.querySelector('[slot=errors]')?.innerHTML,
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-division')
  dom.draggable = true
  dom.id = node.attrs.id
  dom.setAttribute('programming-language', node.attrs.programmingLanguage)
  if (node.attrs.guessLanguage)
    dom.setAttribute('guess-language', node.attrs.guessLanguage)

  const code = document.createElement('pre')
  code.slot = 'code'
  code.innerHTML = node.attrs.code
  code.contentEditable = 'false'
  dom.appendChild(code)

  const css = document.createElement('pre')
  css.slot = 'css'
  css.innerHTML = node.attrs.css
  css.contentEditable = 'false'
  dom.appendChild(css)

  const errors = document.createElement('div')
  errors.slot = 'errors'
  errors.innerHTML = node.attrs.errors
  errors.contentEditable = 'false'
  dom.appendChild(errors)

  const contentDOM = document.createElement('div')
  contentDOM.slot = 'content'
  dom.appendChild(contentDOM)

  return { dom, contentDOM }
}
