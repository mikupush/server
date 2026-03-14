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

import FileDetails from "@/components/FileDetails.tsx";
import {useEffect, useState} from "react";
import type {FileInfoError, FileInfo, UploadMetadata} from "@/models.ts";
import {useTranslation} from "react-i18next";
import {Button} from "@/components/ui/button.tsx";
import {Download, LoaderCircle, TriangleAlert} from "lucide-react";

export default function FileDownload() {
  const { t } = useTranslation()
  const [details, setDetails] = useState<FileInfo | null>(null)
  const [isLoading, setIsLoading] = useState<boolean>(true)
  const [error, setError] = useState<string>('')

  useEffect(() => {

    setIsLoading(true)
    fetchDetails()
      .then(info => setDetails(info))
      .catch(error => setError(error))
      .finally(() => setIsLoading(false))
  }, [])

  const fetchDetails = async () => {
    const metadata: UploadMetadata = JSON.parse(document.getElementById('upload-metadata')!.innerText)
    const response = await fetch(`/api/file/${metadata.id}`)

    if (response.status !== 200) {
      const error = await response.json() as FileInfoError
      throw error.code
    }

    return await response.json() as FileInfo
  }

  const View = () => (
    <div className="flex flex-col items-center">
      <FileDetails details={details!} />
      <Button asChild className="mt-6 p-6 text-lg">
        <a href={`/u/${details.id}`} download>
          <Download className="size-5" />
          {t('download_file')}
        </a>
      </Button>
    </div>
  )

  const Loading = () => (
    <LoaderCircle className="mx-auto animate-spin" size={100} />
  )

  const Error = () => {
    const getErrorMessage = (errorCode: string) => {
      switch (errorCode) {
        case 'NotExists':
          return t('file_not_found')
        case 'InvalidPathParameter':
          return t('invalid_file_id')
        default:
          return t('system_error')
      }
    }

    return (
      <div className="flex flex-col items-center">
        <TriangleAlert className="text-red-500" size={100} />
        <p className="text-2xl mt-3 px-5 text-center">{getErrorMessage(error)}</p>
      </div>
    )
  }

  return (
    <>
      {(isLoading) ? (
        <Loading />
      ) : (error) ? (
        <Error />
      ) : (
        <View />
      )}
    </>
  )
}