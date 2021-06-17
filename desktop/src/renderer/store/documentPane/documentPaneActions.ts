import { EntityId } from '@reduxjs/toolkit'
import { option as O } from 'fp-ts'
import { Document } from 'stencila'
import { CHANNEL } from '../../../preload/channels'
import { store } from '../index'
import { documentPaneActions } from './documentPaneStore'

export const initPane = (paneId: EntityId) => {
  store.dispatch(documentPaneActions.createPane({ paneId }))
}

export const addDocumentToPane = async (paneId: EntityId, docId: EntityId) => {
  const document = (await window.api.invoke(
    CHANNEL.OPEN_DOCUMENT,
    docId
  )) as Document

  return store.dispatch(
    documentPaneActions.addDocToPane({
      paneId,
      view: { type: 'editor', ...document },
    })
  )
}

export const closeDocument = (paneId: EntityId, docId: EntityId) => {
  store.dispatch(
    documentPaneActions.removeDocFromPane({
      paneId,
      docId,
    })
  )
}

export const setActiveDocument = (paneId: EntityId, docId: EntityId) => {
  store.dispatch(
    documentPaneActions.updatePane({
      id: paneId,
      changes: { activeView: O.some(docId) },
    })
  )
}
