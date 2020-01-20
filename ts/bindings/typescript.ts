/**
 * Generate Typescript language bindings.
 */

import fs from 'fs-extra'
import camelCase from 'lodash/camelCase'
import path from 'path'
import prettier from 'prettier'
import {
  autogeneratedHeader,
  filterTypeSchemas,
  filterUnionSchemas,
  getSchemaProperties,
  readSchemas,
  Schema
} from '../helpers'

/**
 * Runs Prettier to beautify code contents based on the project settings
 */
const prettify = async (contents: string): Promise<string> => {
  const config = await prettier
    .resolveConfigFile()
    .then(path => (path !== null ? prettier.resolveConfig(path) : undefined))

  return prettier.format(
    contents,
    config !== null ? { ...config, parser: 'typescript' } : undefined
  )
}

/**
 * Generate `../types.ts` from schemas.
 */
export const generateTypeDefinitions = async (): Promise<string> => {
  const schemas = await readSchemas()

  const typesCode = filterTypeSchemas(schemas)
    .map(typeGenerator)
    .join('')

  const unionsCode = filterUnionSchemas(schemas)
    .map(unionGenerator)
    .join('')

  const code = `/* eslint-disable */

${autogeneratedHeader('build:ts', path.basename(__filename), '//')}

// Remove properties from an Object if their values is undefined
const compact = <O extends object>(o: O): O =>
  Object.entries(o).reduce(
    (compactedO: O, [k, v]) =>
      v === undefined ? compactedO : { ...compactedO, [k]: v },
    {} as O
  )

${typesInterface(schemas)}

${typesCode}

${unionsCode}
`

  const file = path.join(__dirname, '..', 'types.ts')
  await fs.writeFile(file, await prettify(code))

  return file
}

/**
 * Generate a `interface Types`, that maps all types
 * and can be used to get a type from its name at compile time.
 */
export const typesInterface = (schemas: Schema[]): string => {
  return `export interface Types {\n${schemas
    .map(({ title }) => `  ${title}: ${title}`)
    .join('\n')}\n}`
}

/**
 * Generate a `interface` and a factory function for each type.
 */
export const typeGenerator = (schema: Schema): string => {
  const {
    title = 'Undefined',
    extends: parent,
    properties,
    description
  } = schema
  const { own, required } = getSchemaProperties(schema)

  const type =
    properties !== undefined
      ? properties.type !== undefined
        ? properties.type.enum !== undefined
          ? properties.type.enum.map(type => `'${type}'`).join(' | ')
          : ''
        : ''
      : ''

  let code = ''

  // Interface
  code += docComment(description)
  code += `export interface ${title} ${
    parent !== undefined ? `extends ${parent}` : ''
  } {\n`
  code += `  type: ${type}\n`
  code += own
    .map(
      ({ name, schema, optional }) =>
        `  ${name}${optional ? `?` : ''}: ${schemaToType(schema)}`
    )
    .join('\n')
  code += '\n}\n\n'

  // Factory function
  code += docComment(`Create a \`${title}\` node`, [
    `@param props Object containing ${title} schema properties as key/value pairs`,
    `@returns {${title}} ${title} schema node`
  ])
  code += `export const ${funcName(title)} = (\n`
  const propsType = `Omit<${title}, 'type'>`
  const propsDefault = required.length <= 0 ? ' = {}' : ''
  code += `  props: ${propsType}${propsDefault}\n`
  code += `): ${title} => ({\n`
  code += `  ...compact(props),\n`
  code += `  type: '${title}'\n`
  code += '})\n\n'

  return code
}

/**
 * Generate a `Union` type.
 */
export const unionGenerator = (schema: Schema): string => {
  const { title, description } = schema
  let code = docComment(description)
  code += `export type ${title} = ${schemaToType(schema)}\n\n`
  return code
}

/**
 * Generate factory function name
 */
const funcName = (name: string): string => {
  const func = `${name.substring(0, 1).toLowerCase() + name.substring(1)}`
  const reserved: { [key: string]: string } = {
    delete: 'del',
    function: 'function_'
  }
  if (reserved[func] !== undefined) return reserved[func]
  else return func
}

/**
 * Generate a JSDoc style comment
 */
const docComment = (description?: string, tags: string[] = []): string => {
  description = description !== undefined ? description : ''
  return (
    '/**\n' +
    ' * ' +
    description.trim().replace('\n', '\n * ') +
    '\n' +
    tags.map(tag => ' * ' + tag.trim().replace('\n', ' ') + '\n').join('') +
    ' */\n'
  )
}

/**
 * Convert a schema definition to a Typescript type
 */
const schemaToType = (schema: Schema): string => {
  const { type, anyOf, allOf, $ref } = schema

  if ($ref !== undefined) return `${$ref.replace('.schema.json', '')}`
  if (anyOf !== undefined) return anyOfToType(anyOf)
  if (allOf !== undefined) return allOfToType(allOf)
  if (schema.enum !== undefined) return enumToType(schema.enum)

  if (type === 'null') return 'null'
  if (type === 'boolean') return 'boolean'
  if (type === 'number') return 'number'
  if (type === 'integer') return 'number'
  if (type === 'string') return 'string'
  if (type === 'array') return arrayToType(schema)
  if (type === 'object') return '{[key: string]: any}'

  throw new Error(`Unhandled schema: ${JSON.stringify(schema)}`)
}

/**
 * Convert a schema with the `anyOf` property to a Typescript `Union` type.
 */
const anyOfToType = (anyOf: Schema[]): string => {
  const types = anyOf
    .map(schema => schemaToType(schema))
    .reduce(
      (prev: string[], curr) => (prev.includes(curr) ? prev : [...prev, curr]),
      []
    )
  if (types.length === 0) return ''
  if (types.length === 1) return types[0]
  return types.join(' | ')
}

/**
 * Convert a schema with the `allOf` property to a Typescript type.
 *
 * If the `allOf` is singular then just use that (this usually arises
 * because the `allOf` is used for a property with a `$ref`). Otherwise,
 * use the last schema (this is usually because one or more codecs can be
 * used on a property and the last schema is the final, expected, type of
 * the property).
 */
const allOfToType = (allOf: Schema[]): string => {
  if (allOf.length === 1) return schemaToType(allOf[0])
  else return schemaToType(allOf[allOf.length - 1])
}

/**
 * Convert a schema with the `array` property to a Typescript `Array` type.
 *
 * Uses the more explicity `Array<>` syntax over the shorter`[]` syntax
 * because the latter necessitates the use of, sometime superfluous, parentheses.
 */
const arrayToType = (schema: Schema): string => {
  const items = Array.isArray(schema.items)
    ? anyOfToType(schema.items)
    : schema.items !== undefined
    ? schemaToType(schema.items)
    : 'any'
  return `Array<${items}>`
}

/**
 * Convert a schema with the `enum` property to Typescript "or values".
 */
export const enumToType = (enu: (string | number)[]): string => {
  return enu
    .map(schema => {
      return JSON.stringify(schema)
    })
    .join(' | ')
}

/**
 * Generate Type Maps for TypeScript type guards and runtime validation
 */
export const generateTypeMaps = async (): Promise<string> => {
  const files = await readSchemas([
    path.join(__dirname, '..', '..', 'public', '*Types.schema.json'),
    path.join(__dirname, '..', '..', 'public', 'BlockContent.schema.json'),
    path.join(__dirname, '..', '..', 'public', 'InlineContent.schema.json')
  ])

  let typeMaps = `
  import * as types from '../types'
  import { TypeMap } from './type-map'

  type Primitives = undefined | null | boolean | string | number;
  `

  files.map(file => {
    // `BlockContent` & `InlineContent` schema dont have a `*Types.schema.json` file
    // This standardizes the TypeMap names so that they all end with `Types`.
    const schemaClass = file.title?.endsWith('Types')
      ? file.title
      : `${file.title}Types`

    typeMaps += `
      export const ${camelCase(schemaClass)}: TypeMap<Exclude<types.${
      file.title
    }, Primitives>> = {
        ${file.anyOf
          ?.reduce((typeMap: string[], type) => {
            const typeRef = type.$ref?.replace('.schema.json', '')
            const typeName = JSON.stringify(typeRef)

            return typeRef !== undefined
              ? [...typeMap, `${typeName}: ${typeName},`]
              : typeMap
          }, [])
          .join('\n')}
        }
      `
  })

  const file = path.join(__dirname, '..', 'util', 'type-maps.ts')
  await fs.writeFile(file, await prettify(typeMaps))

  return file
}

/** Generate Type Definitions and Type Maps files */
export const build = async (): Promise<unknown> => {
  return generateTypeDefinitions().then(generateTypeMaps)
}

/**
 * Run `build()` when this file is run as a Node script
 */
// eslint-disable-next-line @typescript-eslint/no-floating-promises
if (module.parent === null) build()
