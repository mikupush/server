/**
 * Miku Push! Server is the backend behind Miku Push!
 * Copyright (C) 2025  Miku Push! Team
 * 
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 * 
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 * 
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

export const DARK_THEME = 'dark'
export const LIGHT_THEME = 'light'
export const THEME_KEY = 'theme'

const store = typeof localStorage !== 'undefined' ? localStorage : {
  getItem: () => LIGHT_THEME,
  setItem: () => {}
}

export function getCurrentTheme() {
  const detectSystemTheme = () => {
    const query = window.matchMedia('(prefers-color-scheme: dark)')
    return query.matches ? DARK_THEME : LIGHT_THEME
  }

  return store.getItem(THEME_KEY) ?? detectSystemTheme()
}

export function toggleDarkTheme() {
  const currentTheme = getCurrentTheme()
  const theme = currentTheme === DARK_THEME ? LIGHT_THEME : DARK_THEME

  applyTheme(theme)
  return theme
}

export function applyTheme(theme: string) {
  if (theme === DARK_THEME) {
    document.documentElement.classList.toggle('dark', true)
  } else {
    document.documentElement.classList.toggle('dark', false)
  }

  store.setItem(THEME_KEY, theme)
}