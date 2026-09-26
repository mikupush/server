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

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {Languages} from "lucide-react";
import {useTranslation} from "react-i18next";

export function LanguageSelect() {
  const {t} = useTranslation()
  const lang = document.documentElement.lang;

  const languages = [
    {value: "en", label: "English"},
    {value: "es", label: "Español"},
  ]

  const selectLanguage = (value: string | null) => {
    if (value === null) return
    document.cookie = `language=${value}`
    window.location.reload()
  }

  return (
    <Select items={languages} value={lang} onValueChange={selectLanguage}>
      <SelectTrigger
        className="border-0 dark:bg-transparent shadow-none hover:bg-muted hover:text-accent-foreground dark:hover:bg-muted/50 gap-3"
        aria-label={t('select_language')}
      >
        <Languages className="h-4"/>
        <SelectValue placeholder={t('select_language')}/>
      </SelectTrigger>
      <SelectContent>
        {languages.map((language) => (
          <SelectItem key={language.value} value={language.value}>{language.label}</SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}
