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

import i18n, {type InitOptions} from 'i18next'
import { initReactI18next } from 'react-i18next'
import en from '@/i18n/en.json'
import es from '@/i18n/es.json'

const options: InitOptions = {
  supportedLngs: ['en', 'es'],
  // the translations
  // (tip move them in a JSON file and import them,
  // or even better, manage them via a UI: https://react.i18next.com/guides/multiple-translation-files#manage-your-translations-with-a-management-gui)
  resources: {
    en: {
      translation: en
    },
    es: {
      translation: es
    }
  },
  lng: 'en', // if you're using a language detector, do not define the lng option
  fallbackLng: 'en',

  interpolation: {
    escapeValue: false // react already safes from xss => https://www.i18next.com/translation-function/interpolation#unescape
  }
}

i18n
  .use(initReactI18next) // passes i18n down to react-i18next
  .init(options);

/**
 * Returns all language paths
 */
export function languagePaths() {
  const paths = [];

  if (typeof options.supportedLngs === 'undefined' || !options.supportedLngs) {
    return paths;
  }

  for (const locale of options.supportedLngs) {
    paths.push({ params: { locale } })
  }

  return paths;
}