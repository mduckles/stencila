import { Component } from 'substance'
import CodeEditorComponent from '../../ui/CodeEditorComponent'
import Cell from './Cell'
import CellValueComponent from './CellValueComponent'
import MiniLangEditor from './MiniLangEditor'
import CellErrorDisplay from './CellErrorDisplay'

class CellComponent extends Component {

  constructor(...args) {
    super(...args)

    this.handleActions({
      // triggered by CodeEditorComponent and MiniLangEditor
      'execute': this._onExecute,
      'break': this._onBreak
    })
  }

  getInitialState() {
    return {
      showCode: true,
      forceShowOutput: false
    }
  }

  /*
    Generally output is shown when cell is not a definition, however it can be
    enforced
  */
  _showOutput() {
    return !this.props.node.isDefinition() || this.state.forceShowOutput
  }

  didMount() {
    const node = this.props.node
    const editorSession = this.context.editorSession
    editorSession.on('render', this.onCellChanged, this, {
      resource: 'document',
      path: [node.id]
    })
    node.on('evaluation:awaiting', this.onAwaitingEvaluation, this)
  }

  dispose() {
    const editorSession = this.context.editorSession
    editorSession.off(this)
  }

  render($$) {
    const cell = this.props.node
    let el = $$('div').addClass('sc-cell')

    let toggleCell = $$('div').addClass('se-toggle-cell').append(
      $$('div').addClass('se-toggle-cell-inner'),
      this._renderMenu($$)
    )

    if (cell.hasErrors()) {
      toggleCell.addClass('sm-error')
    }
    el.append(toggleCell)

    if (this.state.showCode) {
      let cellEditorContainer = $$('div').addClass('se-cell-editor-container')
      cellEditorContainer.append(
        $$('div').addClass('se-expression').append(
          $$(MiniLangEditor, {
            path: [cell.id, 'expression'],
            commands: ['undo', 'redo', 'select-all'],
            expression: cell.getExpressionNode()
          }).ref('expressionEditor')
        )
      )

      if (cell.isExternal()) {
        cellEditorContainer.append(
          $$(CodeEditorComponent, {
            path: [cell.id, 'sourceCode'],
            language: cell.context
          }).ref('sourceCodeEditor')
            .on('escape', this.onEscapeFromCodeEditor)
        )
      }

      el.append(cellEditorContainer)
      el.append(
        $$(CellErrorDisplay, {cell})
      )
    }

    if (this._showOutput()) {
      el.append(
        $$(CellValueComponent, {cell}).ref('value')
      )
    }
    return el
  }

  _toggleShowCode(event) {
    event.preventDefault()
    event.stopPropagation()
    this.extendState({
      showCode: !this.state.showCode
    })
  }

  _toggleForceShowOutput(event) {
    event.preventDefault()
    event.stopPropagation()
    let isDefinition = this.props.node.isDefinition()
    // No toggling allowed if cell is not a definition
    if (!isDefinition) return
    console.log('TOGGLING forceShowOutput')
    this.extendState({
      forceShowOutput: !this.state.forceShowOutput
    })
  }

  _renderMenu($$) {
    let menuEl = $$('div').addClass('se-menu')
    menuEl.append(
      this._renderToggleCode($$),
      this._renderToggleOutput($$)
    )
    return menuEl
  }

  /*
    Displays 'Show Code' or 'Hide Code' depending on the current state
  */
  _renderToggleCode($$) {
    let el = $$('div')
      .addClass('se-menu-item')
      .on('click', this._toggleShowCode)

    if (this.state.showCode) {
      el.append('Hide Code')
    } else {
      el.append('Show Code')
    }
    return el
  }

  _renderToggleOutput($$) {
    let el = $$('div')
      .addClass('se-menu-item')
      .on('click', this._toggleForceShowOutput)

    // If cell is not a definition we ensure output is always shown
    let isDefinition = this.props.node.isDefinition()
    if (!isDefinition) {
      el.addClass('sm-disabled')
    }

    if (this._showOutput()) {
      el.append('Hide Output')
    } else {
      el.append('Show Output')
    }
    return el
  }

  getExpression() {
    return this.refs.expressionEditor.getContent()
  }

  onEscapeFromCodeEditor(event) {
    event.stopPropagation()
    this.send('escape')
  }

  onContextInputChanged(event) {
    const context = event.target.value
    const cell = this.props.node
    cell.context = context
    Cell.contextDefault = context
    cell.recompute()
    this.rerender()
  }

  onCellChanged() {
    this.rerender()
  }

  onSelectEllipsesDropdown(event) {
    const data = event.detail
    const { name, value } = data
    if (name) {
      let newState = {}
      newState[name] = value
      this.extendState(newState)
    } else {
      console.error('FIXME: illegal event emitted by Dropdown')
    }
  }

  onAwaitingEvaluation() {
    // TODO: we could visualize this
    // TODO: we could freeze the editor so that no further evaluations
    // are triggered by typing; this might as well depend on the
  }

  _onExecute() {
    this.props.node.recompute()
  }

  _onBreak() {
    this.context.editorSession.transaction((tx) => {
      tx.selection = this._afterNode()
      tx.insertBlockNode({
        type: 'paragraph'
      })
    })
  }

  _afterNode() {
    // TODO: not too happy about how difficult it is
    // to set the selection
    const node = this.props.node
    const isolatedNode = this.context.isolatedNodeComponent
    const parentSurface = isolatedNode.getParentSurface()
    return {
      type: 'node',
      nodeId: node.id,
      mode: 'after',
      containerId: parentSurface.getContainerId(),
      surfaceId: parentSurface.id
    }
  }

}

CellComponent.noBlocker = true

export default CellComponent
