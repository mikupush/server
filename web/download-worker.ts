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

context.onmessage = async function (event) {
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
  const { url, size } = event.data;

  let directory: FileSystemDirectoryHandle | undefined;
  let access: FileSystemSyncAccessHandle | undefined;
  let reader: ReadableStreamDefaultReader<Uint8Array> | undefined;
  let name: string | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let complete = false;

  function refreshTimeout() {
    const timeoutMs = 120_000;

    clearTimeout(timer);
    timer = setTimeout(function () {
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

    reader = response.body.getReader();
    let received = 0;
    let lastProgress = 0;

    while (true) {
      refreshTimeout();
      const { done, value } = await reader.read();
      abortController.signal.throwIfAborted();

      if (done) {
        break;
      }

      if (received + value.byteLength > size) {
        throw new Error('Unexpected file size');
      }

      let offset = 0;

      while (offset < value.byteLength) {
        const written = access.write(value.subarray(offset), { at: received + offset });

        if (written <= 0) {
          throw new Error('Unable to write file');
        }

        offset += written;
      }

      received += value.byteLength;

      if (Date.now() - lastProgress >= progressIntervalMs || received === size) {
        lastProgress = Date.now();
        context.postMessage({ type: 'progress', received });
      }
    }

    if (received !== size) {
      throw new Error('Unexpected file size');
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
