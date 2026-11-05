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

/// <reference lib="webworker" />

import {createDecryptionStream} from './cipher';

type DownloadRequest =
  | { type: 'start'; url: string; size: number; encryptionKey?: string }
  | { type: 'cancel' };

const context = self as unknown as DedicatedWorkerGlobalScope;
let abortController: AbortController | undefined;
let started = false;

async function removeOldDownloads(directory: FileSystemDirectoryHandle) {
  const maxFileAgeMs = 24 * 60 * 60 * 1000;
  const temporaryFilePattern = /^\d+-[a-f0-9-]+$/;

  for await (const [name, handle] of directory.entries()) {
    if (handle.kind !== 'file' || !temporaryFilePattern.test(name)) {
      continue;
    }

    try {
      const fileHandle = handle as FileSystemFileHandle;
      const file = await fileHandle.getFile();

      if (Date.now() - file.lastModified < maxFileAgeMs) {
        continue;
      }

      // A live writer holds an exclusive lock; never remove its temporary file.
      const access = await fileHandle.createSyncAccessHandle();
      access.close();
      await directory.removeEntry(name);
    } catch {
      // Another tab may still own the file, or may have removed it already.
    }
  }
}

function getDownloadErrorCode(error: unknown) {
  if (error instanceof DOMException && error.name === 'QuotaExceededError') {
    return 'storage';
  }

  if (error instanceof Error && error.message === 'unsupported') {
    return 'unsupported';
  }

  return 'failed';
}

context.onmessage = async (event: MessageEvent<DownloadRequest>) => {
  const progressIntervalMs = 150;

  if (event.data.type === 'cancel') {
    abortController?.abort();
    return;
  }

  if (event.data.type !== 'start' || started) {
    return;
  }

  started = true;
  abortController = new AbortController();
  const signal = abortController.signal;
  const { url, size, encryptionKey } = event.data;

  let directory: FileSystemDirectoryHandle | undefined;
  let access: FileSystemSyncAccessHandle | undefined;
  let reader: ReadableStreamDefaultReader<Uint8Array> | undefined;
  let name: string | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let complete = false;

  function refreshTimeout() {
    const timeoutMs = 120_000;

    clearTimeout(timer);
    timer = setTimeout(() => {
      abortController?.abort(new Error('Download timed out'));
    }, timeoutMs);
  }

  try {
    if (!navigator.storage?.getDirectory) {
      throw new Error('unsupported');
    }

    if (!Number.isSafeInteger(size) || size < 0) {
      throw new Error('Invalid file size');
    }

    const decryption = encryptionKey === undefined
      ? undefined
      : await createDecryptionStream(encryptionKey);

    const root = await navigator.storage.getDirectory();
    directory = await root.getDirectoryHandle('mikupush-downloads', { create: true });
    await removeOldDownloads(directory);
    abortController.signal.throwIfAborted();

    const estimate = await navigator.storage.estimate();

    if (estimate.quota !== undefined && size > estimate.quota - (estimate.usage ?? 0)) {
      throw new DOMException('Insufficient storage', 'QuotaExceededError');
    }

    name = `${Date.now()}-${crypto.randomUUID()}`;
    const handle = await directory.getFileHandle(name, { create: true });

    if (!handle.createSyncAccessHandle) {
      throw new Error('unsupported');
    }

    access = await handle.createSyncAccessHandle();
    abortController.signal.throwIfAborted();
    refreshTimeout();

    const response = await fetch(url, { signal: abortController.signal, cache: 'no-store' });

    if (!response.ok || !response.body) {
      throw new Error(`HTTP ${response.status}`);
    }

    let received = 0;
    let lastProgress = 0;
    let writtenBytes = 0;

    // Count ciphertext bytes for the server's size check and download progress.
    const download = response.body.pipeThrough(new TransformStream<Uint8Array, Uint8Array>({
      transform: (value: Uint8Array, controller: TransformStreamDefaultController<Uint8Array>) => {
        signal.throwIfAborted();
        refreshTimeout();

        if (received + value.byteLength > size) {
          throw new Error('Unexpected file size');
        }

        received += value.byteLength;

        if (Date.now() - lastProgress >= progressIntervalMs || received === size) {
          lastProgress = Date.now();
          context.postMessage({ type: 'progress', received });
        }

        controller.enqueue(value);
      },
      flush: () => {
        if (received !== size) {
          throw new Error('Unexpected file size');
        }
      }
    }));

    reader = (decryption ? download.pipeThrough(decryption) : download).getReader();

    while (true) {
      refreshTimeout();
      const { done, value } = await reader.read();
      abortController.signal.throwIfAborted();

      if (done) {
        break;
      }

      let offset = 0;

      while (offset < value.byteLength) {
        const written = access.write(value.subarray(offset), { at: writtenBytes + offset });

        if (written <= 0) {
          throw new Error('Unable to write file');
        }

        offset += written;
      }

      writtenBytes += value.byteLength;
    }

    access.flush();
    access.close();
    access = undefined;

    const file = await handle.getFile();
    abortController.signal.throwIfAborted();

    // Keep the backing file for the browser's asynchronous save; reclaim it on a later download.
    context.postMessage({ type: 'complete', file });
    complete = true;
  } catch (error) {
    const code = getDownloadErrorCode(error);
    context.postMessage({ type: 'error', code });
  } finally {
    clearTimeout(timer);
    abortController.abort();

    try {
      await reader?.cancel();
    } catch {}

    reader?.releaseLock();
    access?.close();

    if (!complete && directory && name) {
      try {
        await directory.removeEntry(name);
      } catch {}
    }

    context.postMessage({ type: 'closed' });
  }
};

export {};
