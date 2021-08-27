import { Component, h } from '@stencil/core'
import { Route } from '@stencil/router'
import { option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { state, store } from '../../../store'
import {
  initPane,
  openDocumentInActivePane,
} from '../../../store/documentPane/documentPaneActions'
import { fetchProject } from '../../../store/project/projectActions'
import { getProjectMainFilePath } from '../../../store/project/projectSelectors'
import { SessionsStoreKeys, sessionStore } from '../../../store/sessionStore'
import { ProjectRouter } from '../projectRouter'
import { listenForFileEvents, removeFileEventListener } from './projectEvents'

const rootPaneId = 1

@Component({
  tag: 'app-project-root',
  styleUrl: 'app-project-root.css',
  scoped: true,
})
export class AppProjectRoot {
  private projectPath: string

  async componentWillLoad() {
    pipe(
      sessionStore.get(SessionsStoreKeys.PROJECT_PATH),
      O.alt(() => {
        const decodedPath = decodeURI(
          window.location.pathname.replace('/project', '')
        )
        sessionStore.set(SessionsStoreKeys.PROJECT_PATH)(decodedPath)
        return O.some(decodedPath)
      }),
      O.map((projectPath) => {
        this.projectPath = projectPath
      })
    )

    initPane(rootPaneId)
    await store.dispatch(fetchProject(this.projectPath))
    listenForFileEvents(this.projectPath)

    const mainFile = getProjectMainFilePath(state)
    if (mainFile) {
      openDocumentInActivePane(mainFile)
    }
  }

  disconnectedCallback() {
    removeFileEventListener()
  }

  render() {
    return (
      <div class="projectWindow">
        <split-me
          n={2}
          sizes={[0.2, 0.8]}
          minSizes={[0.05, 0.2]}
          maxSizes={[0.5, 1]}
          d="horizontal"
        >
          <app-project-sidebar-files slot="0"></app-project-sidebar-files>

          <div slot="1">
            <ProjectRouter.Switch>
              <Route
                path={() => true}
                render={() => (
                  <main>
                    <app-document-pane paneId={rootPaneId}></app-document-pane>
                  </main>
                )}
              ></Route>
            </ProjectRouter.Switch>
          </div>
        </split-me>
      </div>
    )
  }
}
