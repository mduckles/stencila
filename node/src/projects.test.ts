import fs from 'fs'
import path from 'path'
import tmp from 'tmp'
import { open, schemas, subscribe, write } from './projects'
import { FileEvent, ProjectEvent } from './types'

function fixture(folder: string) {
  return path.normalize(
    path.join(__dirname, '..', '..', 'fixtures', 'projects', folder)
  )
}

test('schema', () => {
  expect(schemas()[0]).toEqual(
    expect.objectContaining({
      $schema: 'https://json-schema.org/draft/2019-09/schema',
      $id: 'Project',
      title: expect.stringMatching(/^Details of a project$/),
      type: 'object',
      properties: expect.objectContaining({
        name: expect.objectContaining({
          description: 'The name of the project',
        }),
      }),
    })
  )
})

test('open: empty', () => {
  let folder = fixture('empty')
  expect(open(folder)).toEqual(
    expect.objectContaining({
      path: folder,
      name: 'empty',
      theme: 'stencila',
    })
  )
})

test('open: manifest', () => {
  let folder = fixture('manifest')
  expect(open(folder)).toEqual(
    expect.objectContaining({
      path: folder,
      name: 'A project with a project.json file',
      theme: 'wilmore',
      mainPath: path.join(folder, 'my-main-file.md'),
      files: expect.objectContaining({
        [path.join(folder, 'project.json')]: expect.objectContaining({
          name: 'project.json',
          format: expect.objectContaining({ name: 'json' }),
        }),
        [path.join(folder, 'my-main-file.md')]: expect.objectContaining({
          name: 'my-main-file.md',
          format: expect.objectContaining({ name: 'md' }),
        }),
      }),
    })
  )
})

/**
 * Test of a workflow involving opening and modifying a project
 */
test('workflow: open and modify', async () => {
  const folder = tmp.dirSync().name
  let projectEvents: ProjectEvent[] = []
  let fileEvents: FileEvent[] = []

  // Open the project
  const project = open(folder)
  expect(project).toEqual(
    expect.objectContaining({
      path: folder,
      theme: 'stencila',
    })
  )

  // Subscribe to the project
  subscribe(folder, ['props'], (_topic, event) =>
    projectEvents.push(event as ProjectEvent)
  )
  subscribe(folder, ['files'], (_topic, event) =>
    fileEvents.push(event as FileEvent)
  )

  // Modify the project.json file on disk
  projectEvents = []
  fileEvents = []
  fs.writeFileSync(
    path.join(folder, 'project.json'),
    JSON.stringify({
      theme: 'wilmore',
    })
  )
  // This timeout needs to be longer than the file watcher debouncing
  await new Promise((resolve) => setTimeout(resolve, 500))
  expect(projectEvents).toEqual([
    expect.objectContaining({
      type: 'updated',
      project: expect.objectContaining({
        path: folder,
        theme: 'wilmore',
      }),
    }),
  ])
  expect(fileEvents).toEqual([
    expect.objectContaining({
      type: 'created',
      path: path.join(folder, 'project.json'),
    }),
  ])
})
