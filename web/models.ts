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

export interface UploadMetadata {
  id: string
}

export type FileInfoErrorCode = 'NotExists' | 'DB' | 'IO' | 'InvalidPathParameter';

export interface FileInfoError {
  code: FileInfoErrorCode | string;
  message: string;
}

export type FileInfoStatus = 'WaitingForUpload' | 'Uploaded';

export interface FileInfo {
  id: string;
  name: string;
  mime_type: string;
  size: number;
  status: FileInfoStatus;
  uploaded_at: string;
  expires_at?: string | null;
  chunked: boolean;
}

export interface HealthStatus {
  status: 'up' | 'down';
}
