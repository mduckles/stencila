import { MenuItemConstructorOptions } from 'electron'
import { showSettings } from '../../config/window'
import { isWindows } from '../../menu/utils'
import { closeActiveTab } from '../../window/windowUtils'
import { openProject } from '../handlers'

export const projectFileMenu: MenuItemConstructorOptions = {
  label: 'File',
  submenu: [
    {
      label: 'Open…',
      accelerator: 'CommandOrControl+o',
      click: () => {
        openProject()
      },
    },
    { type: 'separator' },
    {
      label: 'Close Tab' as const,
      click: () => {
        closeActiveTab()
      },
      accelerator: isWindows ? 'Control+Shift+W' : 'CommandOrControl+w',
    },
    {
      role: 'close' as const,
      accelerator: isWindows ? 'Alt+F4' : 'CommandOrControl+Shift+w',
    },
    { type: 'separator' as const },
    {
      label: 'Preferences…',
      accelerator: 'CommandOrControl+,',
      click: () => {
        showSettings()
      },
    },
    { type: 'separator' as const },
    { role: 'quit' as const },
  ],
}
