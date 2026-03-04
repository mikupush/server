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
}

export interface HealthStatus {
  status: 'up' | 'down';
}
