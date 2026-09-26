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

import type {FileInfo} from './models';

export async function prepareDownload() {
  if (!('serviceWorker' in navigator)) {
    return;
  }

  const registrations = await navigator.serviceWorker.getRegistrations();

  for (const registration of registrations) {
    const worker = registration.active ?? registration.waiting ?? registration.installing;

    if (worker && /\/service-worker(?:-[\w-]+)?\.(?:js|ts)$/.test(new URL(worker.scriptURL).pathname)) {
      await registration.unregister();
    }
  }
}

export function startDownload(
  file: FileInfo,
  onProgress: (bytes: number) => void
): { done: Promise<void>; cancel: () => void } {
  let worker: Worker | undefined;
  let settled = false;
  let rejectDownload: (reason: Error) => void;

  function beforeUnload(event: BeforeUnloadEvent) {
    event.preventDefault();
    event.returnValue = true;
  }

  function cancel() {
    if (settled) {
      return;
    }

    settled = true;
    worker?.postMessage({ type: 'cancel' });
    rejectDownload(new Error('cancelled'));
  }

  const done = new Promise<void>(function (resolve, reject) {
    rejectDownload = reject;

    if (!window.isSecureContext || !('Worker' in window) || !navigator.storage?.getDirectory) {
      reject(new Error('unsupported'));
      return;
    }

    window.addEventListener('beforeunload', beforeUnload);
    window.addEventListener('pagehide', cancel);

    worker = new Worker(new URL('./download-worker.ts', import.meta.url), { type: 'module' });

    function failWorker() {
      worker?.terminate();
      reject(new Error('failed'));
    }

    worker.onerror = failWorker;
    worker.onmessageerror = failWorker;

    worker.onmessage = function (event) {
      if (event.data.type === 'closed') {
        worker?.terminate();
        return;
      }

      if (settled) {
        return;
      }

      if (event.data.type === 'progress') {
        onProgress(event.data.received);
      }

      if (event.data.type === 'error') {
        reject(new Error(event.data.code));
      }

      if (event.data.type !== 'complete') {
        return;
      }

      try {
        const url = URL.createObjectURL(event.data.file);

        const link = document.createElement('a');
        link.href = url;
        link.download = file.name;
        link.hidden = true;
        document.body.append(link);

        window.removeEventListener('beforeunload', beforeUnload);
        link.click();
        link.remove();

        // Do not revoke while the browser is starting the save dialog.
        setTimeout(function () {
          URL.revokeObjectURL(url);
        }, 60000);

        resolve();
      } catch {
        reject(new Error('failed'));
      }
    };

    worker.postMessage({ type: 'start', url: `/u/${file.id}?raw`, size: file.size });
  }).finally(function () {
    settled = true;
    window.removeEventListener('beforeunload', beforeUnload);
    window.removeEventListener('pagehide', cancel);
  });

  return { done, cancel };
}

export function formatDownloadSpeed(speed: number): string {
  const speedUnits = ['B/s', 'KiB/s', 'MiB/s', 'GiB/s']
  const speedUnit = speed > 0 ? Math.min(3, Math.floor(Math.log(speed) / Math.log(1024))) : 0
  const unit = Math.max(0, speedUnit)

  return `${(speed / 1024 ** unit).toFixed(1)} ${speedUnits[unit]}`
}
