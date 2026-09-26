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

import {useState} from "react";
import {useTranslation} from "react-i18next";
import {ArrowDownToLine, CircleCheck, CircleSlash, CircleX, Download, LoaderCircle, X} from "lucide-react";
import {Button} from "@/components/ui/button.tsx";
import {Progress} from "@/components/ui/progress.tsx";
import {Drawer, DrawerContent, DrawerHeader, DrawerTitle, DrawerDescription} from "@/components/ui/drawer.tsx";
import type {FileInfo} from "@/models.ts";
import {useDownload} from "@/hooks/useDownload";
import {formatDownloadSpeed} from "@/download.ts";

interface DownloadProgressProps {
  details: FileInfo;
}

export default function DownloadProgress({details}: DownloadProgressProps) {
  const {t} = useTranslation()
  const [open, setOpen] = useState(false)
  const {status, received, speed, error, cancelDownload, retryDownload} = useDownload(details)
  const downloading = status === 'downloading'
  const percent = details.size ? Math.min(100, Math.floor(received / details.size * 100)) : 0

  const downloadFile = () => {
    if (downloading) {
      return
    }

    retryDownload()
    setOpen(true)
  }

  const showProgress = () => {
    setOpen(true)
  }

  const hideProgress = () => {
    setOpen(false)
  }

  return (
    <>
      <Button className="p-6 text-lg" onClick={downloadFile} disabled={downloading} aria-busy={downloading}>
        {downloading ? (
          <LoaderCircle className="size-5 animate-spin" aria-hidden="true" />
        ) : (
          <Download className="size-5" aria-hidden="true" />
        )}
        {t('download_file')}
      </Button>

      {status !== 'idle' && !open && (
        <Button variant="outline" className="fixed bottom-4 right-4 z-40 shadow-md" onClick={showProgress}>
          {t('view_download')}
        </Button>
      )}

      <Drawer
        open={open}
        onOpenChange={setOpen}
        swipeDirection="down"
        modal={false}
        disablePointerDismissal
      >
        <DrawerContent>
          <div className="mx-auto w-full max-w-xl px-4 pb-6">
            <DrawerHeader>
              <DrawerTitle className="break-all">{t('downloading_file')}</DrawerTitle>
              <DrawerDescription className="text-center">{t('download_keep_open')}</DrawerDescription>
              <Button
                variant="ghost"
                size="icon"
                className="absolute right-2 top-2"
                aria-label={t('hide_download')}
                onClick={hideProgress}
              >
                <X className="size-5" />
              </Button>
            </DrawerHeader>

            {downloading && (
              <div className="flex space-x-3 my-5">
                <div className="flex justify-center items-center rounded bg-blue-100 text-blue-600 size-20">
                  <ArrowDownToLine size="30" />
                </div>
                <div className="flex-1 space-y-2">
                  <p className="font-bold line-clamp-1">{details.name}</p>
                  <div className="flex flex-wrap justify-between gap-2 text-sm tabular-nums">
                    <p>{t('download_progress', {percent})}</p>
                    <p>{t('download_speed', {speed: formatDownloadSpeed(speed)})}</p>
                  </div>
                  <Progress value={percent} aria-label={t('download')} />
                  <div className="flex justify-end">
                    <Button variant="outline" onClick={cancelDownload}>{t('cancel_download')}</Button>
                  </div>
                </div>
              </div>
            )}

            {status === 'completed' && (
              <div className="flex items-center justify-center my-5">
                <CircleCheck size="30" className="text-green-600" />
                <p role="status" className="ml-2">{t('download_complete')}</p>
              </div>
            )}

            {status === 'cancelled' && (
              <div className="flex items-center justify-center my-5">
                <CircleSlash size="30" />
                <p role="status" className="ml-2">{t('download_cancelled')}</p>
              </div>
            )}

            {status === 'error' && (
              <div className="flex items-center justify-center my-5">
                <CircleX size="30" className="text-destructive" />
                <p role="status" className="text-destructive ml-2">{t(error)}</p>
              </div>
            )}
          </div>
        </DrawerContent>
      </Drawer>
    </>
  )
}
