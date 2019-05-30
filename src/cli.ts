import yargs from 'yargs'
import { logger } from './logging'

import { addCliCommands as addEncodaCliCommands } from './encoda'

const VERSION = require('../package').version

//@ts-ignore
process.on('stencila-log', (level: string, message: string) => {
  logger.log(level, message)
})

const yargsDefinition = yargs.scriptName('stencila')

addEncodaCliCommands(yargsDefinition, cleanup)
  // Any command-line argument given that is not demanded, or does not have a corresponding description, will be reported as an error.
  // Unrecognized commands will also be reported as errors.
  .strict()

  // Maximize width of usage instructions
  .wrap(yargs.terminalWidth())

  // Help global option
  .usage('$0 <cmd> [args]')
  .alias('help', 'h')

  // Version global option
  .version(VERSION)
  .alias('version', 'v')
  .describe('version', 'Show version')

  .parse()

function cleanup() {
  // Trigger a clean up
  //   "The 'beforeExit' event is not emitted for conditions causing
  //   explicit termination, such as calling process.exit() or uncaught
  //   exceptions."
  process.emit('beforeExit', 0)
}
