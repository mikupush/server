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

import {useEffect, useState} from "react";
import {useTranslation} from "react-i18next";
import {LoaderCircle, TriangleAlert} from "lucide-react";
import FileDetails from "@/components/FileDetails.tsx";
import DownloadProgress from "@/components/DownloadProgress.tsx";
import {Card, CardContent} from "@/components/ui/card.tsx";
import {Separator} from "@/components/ui/separator.tsx";
import type {FileInfoError, FileInfo, UploadMetadata} from "@/models.ts";

async function fetchFileDetails(signal: AbortSignal): Promise<FileInfo> {
  const metadataElement = document.getElementById('upload-metadata')!
  const metadata: UploadMetadata = JSON.parse(metadataElement.innerText)
  const response = await fetch(`/api/file/${metadata.id}`, {signal})

  if (!response.ok) {
    const error = await response.json() as FileInfoError
    throw error.code
  }

  return await response.json() as FileInfo
}

function getFileErrorMessage(error: string): string {
  switch (error) {
    case 'NotExists':
      return 'file_not_found'
    case 'InvalidPathParameter':
      return 'invalid_file_id'
    default:
      return 'system_error'
  }
}

export default function FileDownload() {
  const {t} = useTranslation()
  const [details, setDetails] = useState<FileInfo | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState('')

  useEffect(() => {
    const abort = new AbortController()

    void fetchFileDetails(abort.signal)
      .then((info) => {
        if (!abort.signal.aborted) {
          setDetails(info)
        }
      })
      .catch((error: unknown) => {
        if (!abort.signal.aborted) {
          setError(String(error))
        }
      })
      .finally(() => {
        if (!abort.signal.aborted) {
          setIsLoading(false)
        }
      })

    return () => {
      abort.abort()
    }
  }, [])

  if (isLoading) {
    return <LoaderCircle className="mx-auto animate-spin" size={100} />
  }

  if (error || !details) {
    return (
      <div className="flex flex-col items-center">
        <TriangleAlert className="text-red-500" size={100} />
        <p className="text-2xl mt-3 px-5 text-center">
          {t(getFileErrorMessage(error))}
        </p>
      </div>
    )
  }

  const isImage = details.mime_type.startsWith('image/')
  const isVideo = details.mime_type.startsWith('video/')
  const canPreview = !details.chunked && (isImage || isVideo)
  const rawUrl = `/u/${details.id}?raw`

  const controls = (
    <div className={canPreview ? 'w-full sm:w-72 sm:ml-6 shrink-0' : 'w-full'}>
      <div className="text-center">
        <DownloadProgress details={details} />
      </div>
    </div>
  )

  if (canPreview) {
    return (
      <Card className="p-0 mx-auto max-w-4xl">
        <CardContent className="p-1">
          <div className="flex justify-center">
            {isImage ? (
              <img className="hidden sm:block rounded-lg max-h-150" src={rawUrl} alt="" />
            ) : (
              <video className="hidden sm:block rounded-lg" src={rawUrl} controls />
            )}
          </div>

          <div className="flex flex-col sm:flex-row justify-between p-3">
            <FileDetails
              details={details}
              variant="row"
              className="hidden sm:block min-w-0"
            />
            <FileDetails
              details={details}
              variant="column"
              className="block sm:hidden"
            />
            <Separator className="block sm:hidden my-6" />
            {controls}
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className="mx-auto max-w-xl">
      <CardContent className="flex flex-col">
        <FileDetails details={details} />
        <Separator className="my-6" />
        {controls}
      </CardContent>
    </Card>
  )
}
