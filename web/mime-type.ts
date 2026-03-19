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

export function isAudioFile(mimeType: string): boolean {
  return mimeType.startsWith('audio/')
}

export function isVideoFile(mimeType: string): boolean {
  return mimeType.startsWith('video/')
}

export function isImageFile(mimeType: string): boolean {
  return mimeType.startsWith('image/')
}

export function isCompressedFile(mimeType: string): boolean {
  const compressedMimeTypes = [
    'application/zip',
    'application/x-zip-compressed',
    'application/x-rar-compressed',
    'application/x-7z-compressed',
    'application/x-tar',
    'application/gzip',
    'application/x-bzip2',
    'application/x-xz',
    'application/x-gtar',
    'application/x-compressed',
    'application/x-gzip',
  ];

  return compressedMimeTypes.includes(mimeType.toLowerCase());
}

export function isWordFile(mimeType: string): boolean {
  const wordMimeTypes = [
    'application/msword',
    'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    'application/vnd.oasis.opendocument.text',
    'application/vnd.oasis.opendocument.text-template',
    'application/vnd.oasis.opendocument.text-web',
    'application/vnd.oasis.opendocument.text-master',
  ];

  return wordMimeTypes.includes(mimeType.toLowerCase());
}

export function isExcelFile(mimeType: string): boolean {
  const excelMimeTypes = [
    'application/vnd.ms-excel',
    'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    'application/vnd.oasis.opendocument.spreadsheet',
    'application/vnd.oasis.opendocument.spreadsheet-template',
  ];

  return excelMimeTypes.includes(mimeType.toLowerCase());
}

export function isPowerPointFile(mimeType: string): boolean {
  const powerPointMimeTypes = [
    'application/vnd.ms-powerpoint',
    'application/vnd.openxmlformats-officedocument.presentationml.presentation',
    'application/vnd.oasis.opendocument.presentation',
    'application/vnd.oasis.opendocument.presentation-template',
  ];

  return powerPointMimeTypes.includes(mimeType.toLowerCase());
}

export function isPlainTextFile(mimeType: string): boolean {
  return mimeType.toLowerCase() === 'text/plain';
}

export function isSourceCodeFile(mimeType: string): boolean {
  const sourceCodeMimeTypes = [
    'text/javascript',
    'application/javascript',
    'text/typescript',
    'text/x-python',
    'text/x-rust',
    'text/x-c',
    'text/x-c++',
    'text/x-java-source',
    'text/x-go',
    'text/html',
    'text/css',
    'application/json',
    'text/markdown',
    'application/x-sh',
    'application/x-php',
    'text/x-ruby',
    'text/xml',
    'application/xml',
  ];

  return sourceCodeMimeTypes.includes(mimeType.toLowerCase());
}

export function isPdfFile(mimeType: string): boolean {
  return mimeType.toLowerCase() === 'application/pdf';
}