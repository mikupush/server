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

import {useEffect, useRef, useState} from "react";
import type {FileInfo} from "@/models.ts";
import {prepareDownload, startDownload} from "@/download";

type DownloadState = {
  status: 'idle' | 'downloading' | 'completed' | 'cancelled' | 'error';
  received: number;
  speed: number;
  error: string;
};

function initialDownloadState(): DownloadState {
  return {status: 'idle', received: 0, speed: 0, error: ''}
}

function downloadErrorState(error: unknown): Partial<DownloadState> {
  if (error instanceof Error && error.message === 'cancelled') {
    return {status: 'cancelled'}
  }
  const messages: Record<string, string> = {
    unsupported: 'download_unsupported', storage: 'download_storage_full'
  }
  return {
    status: 'error',
    error: error instanceof Error ? messages[error.message] ?? 'download_failed' : 'download_failed'
  }
}

function createDownloadSession(details: FileInfo, onChange: (state: Partial<DownloadState>) => void) {
  let disposed = false
  let received = 0
  let previousBytes = 0
  let previousTime = performance.now()
  void prepareDownload().catch(function (error: unknown) {
    console.warn('Unable to unregister the previous download service worker', error)
  })

  const download = startDownload(details, function (bytes) {
    if (disposed) return
    received = bytes
    onChange({received})
  })

  const timer = setInterval(function () {
    const now = performance.now()
    onChange({speed: (received - previousBytes) * 1000 / (now - previousTime)})
    previousBytes = received
    previousTime = now
  }, 1000)

  void download.done.then(function () {
    if (!disposed) onChange({status: 'completed'})
  }, function (error: unknown) {
    if (!disposed) onChange(downloadErrorState(error))
  }).finally(function () {
    clearInterval(timer)
  })

  return {
    cancel: download.cancel,
    dispose() {
      disposed = true
      clearInterval(timer)
      download.cancel()
    }
  }
}

export function useDownload(details: FileInfo) {
  const [state, setState] = useState(initialDownloadState)
  const [attempt, setAttempt] = useState(0)
  const session = useRef<ReturnType<typeof createDownloadSession> | null>(null)

  useEffect(() => {
    if (attempt === 0) {
      return
    }

    setState({...initialDownloadState(), status: 'downloading'})
    const currentSession = createDownloadSession(details, (changes) => {
      setState((previous) => ({...previous, ...changes}))
    })
    session.current = currentSession
    return () => {
      currentSession.dispose()
      session.current = null
    }
  }, [details, attempt])

  const cancelDownload = () => { session.current?.cancel() }
  const retryDownload = () => {
    setState({...initialDownloadState(), status: 'downloading'})
    setAttempt((previous) => previous + 1)
  }

  return {...state, cancelDownload, retryDownload}
}
