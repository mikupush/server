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
import i18n from "i18next";

export function LanguageSelect() {
  const {t} = useTranslation()

  return (
    <Select value={i18n.language}>
      <SelectTrigger
        className="border-0 shadow-transparent hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50 gap-3"
        aria-label={t('select_language')}
      >
        <Languages className="h-4"/>
        <SelectValue placeholder={t('select_language')}/>
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="en">English</SelectItem>
        <SelectItem value="es">Español</SelectItem>
      </SelectContent>
    </Select>
  )
}