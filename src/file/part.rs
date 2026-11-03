// Miku Push! Server is the backend behind Miku Push!
// Copyright (C) 2025  Miku Push! Team
// 
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, warn, info};
use uuid::Uuid;
use crate::cache::MokaCache;
use crate::config::Settings;
use crate::file::{FileUploadRepository, FileUploadRepositoryError, PostgresFileUploadRepository};
use crate::file::lock::{ChunkedReaderLocker, InMemoryChunkedReaderLocker};
use crate::storage::{FileSystemObjectStorageAppender, FileSystemObjectStorageCounter, FileSystemObjectStorageReader, FileSystemObjectStorageRemover, ObjectStorageAppendError, ObjectStorageAppender, ObjectStorageCounter, ObjectStorageReader, ObjectStorageRemoveError, ObjectStorageRemover};

pub struct FilePart;

impl FilePart {
    pub fn name(index: usize) -> String {
        format!("{}.part", index)
    }
}

pub trait PartCleanupJob {
    fn dispatch(&self, id: &Uuid) -> Result<(), String>;
}

#[derive(Clone)]
pub struct FileAssembler<FR, OBR, OBA, PCJ>
where
    FR: FileUploadRepository + Clone,
    OBR: ObjectStorageReader + Clone,
    OBA: ObjectStorageAppender + Clone,
    PCJ: PartCleanupJob + Clone,
{
    settings: Settings,
    repository: FR,
    reader: OBR,
    appender: OBA,
    cleanup_job: PCJ,
}

impl<FR, OBR, OBA, PCJ> FileAssembler<FR, OBR, OBA, PCJ>
where
    FR: FileUploadRepository + Clone,
    OBR: ObjectStorageReader + Clone,
    OBA: ObjectStorageAppender + Clone,
    PCJ: PartCleanupJob + Clone,
{
    pub fn new(settings: &Settings, repository: FR, reader: OBR, appender: OBA, cleanup_job: PCJ) -> Self {
        Self { settings: settings.clone(), repository, reader, appender, cleanup_job }
    }

    pub fn assemble(&self, id: &Uuid) -> Result<(), FileAssembleError> {
        let Some(file_upload) = self.repository.find_by_id(id)? else {
            return Err(FileAssembleError::NotFound(id.clone()));
        };

        let mut file_upload = file_upload;
        if !file_upload.chunked {
            return Err(FileAssembleError::NotChunked(id.clone()));
        }

        let directory = file_upload.content_directory(&self.settings)?;
        let assembled_name = Path::new(&directory).join(&file_upload.name)
            .to_string_lossy()
            .to_string();

        let parts = std::fs::read_dir(&directory)?.count();

        for part in 0..parts {
            let name = FilePart::name(part);
            let location = Path::new(&directory).join(name)
                .to_string_lossy()
                .to_string();

            let read_stream = self.reader.read(&location)?;
            self.appender.append(read_stream, assembled_name.clone())?;
        }

        file_upload.chunked = false;
        self.repository.save(&file_upload)?;

        if let Err(err) = self.cleanup_job.dispatch(id) {
            warn!(upload = ?id, error = err, "failed to equeue file part cleanup job")
        }

        Ok(())
    }
}

impl FileAssembler<
    PostgresFileUploadRepository<MokaCache>,
    FileSystemObjectStorageReader,
    FileSystemObjectStorageAppender,
    FileSystemInMemoryPartCleanupJob
> {
    pub fn get_with_settings(settings: &Settings) -> Self {
        Self {
            settings: settings.clone(),
            repository: PostgresFileUploadRepository::get_with_settings(&settings),
            reader: FileSystemObjectStorageReader::new(),
            appender: FileSystemObjectStorageAppender::new(),
            cleanup_job: FileSystemInMemoryPartCleanupJob::get_instance()
        }
    }
}

#[derive(Debug)]
pub enum FileAssembleError {
    IO(String),
    NotFound(Uuid),
    NotChunked(Uuid),
}

impl From<FileUploadRepositoryError> for FileAssembleError {
    fn from(value: FileUploadRepositoryError) -> Self {
        match value {
            FileUploadRepositoryError::Pool(err) => Self::IO(err.to_string()),
            FileUploadRepositoryError::Db(err) => Self::IO(err.to_string()),
        }
    }
}

impl From<std::io::Error> for FileAssembleError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value.to_string())
    }
}

impl From<ObjectStorageAppendError> for FileAssembleError {
    fn from(value: ObjectStorageAppendError) -> Self {
        match value {
            ObjectStorageAppendError::IO(err) => Self::IO(err.to_string()),
        }
    }
}

impl Display for FileAssembleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(err) => write!(f, "io error: {}", err),
            Self::NotFound(id) => write!(f, "file upload not found: {}", id),
            Self::NotChunked(id) => write!(f, "file upload is not chunked: {}", id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssembledFilePartsCleaner<OBR, FR, OBC>
where
    OBR: ObjectStorageRemover + Clone,
    FR: FileUploadRepository + Clone,
    OBC: ObjectStorageCounter + Clone
{
    remover: OBR,
    repository: FR,
    counter: OBC,
    settings: Settings,
}

impl<OBR, FR, OBC> AssembledFilePartsCleaner<OBR, FR, OBC>
where
    OBR: ObjectStorageRemover + Clone,
    FR: FileUploadRepository + Clone,
    OBC: ObjectStorageCounter + Clone,
{
    pub fn new(remover: OBR, repository: FR, settings: &Settings, counter: OBC) -> Self {
        Self {
            remover,
            repository,
            settings: settings.clone(),
            counter
        }
    }

    pub fn clean(&self, id: &Uuid) -> Result<(), PartCleanupError> {
        let file = match self.repository.find_by_id(id)? {
            Some(file) => file,
            None => return Err(PartCleanupError::NotFound(id.clone()))
        };

        let content_path = file.content_path(&self.settings)?;
        let parts = self.counter.count_in_directory(&content_path.to_string_lossy().to_string())?;

        for part in 0..parts {
            let part_name = FilePart::name(part);
            let path = Path::new(&content_path).join(part_name)
                .to_string_lossy()
                .to_string();

            if let Err(err) = self.remover.remove(path) {
                // ignore not exists error because counter can count the assembled file
                // so it is probably that the remover will try to delete a non-existing part
                if let ObjectStorageRemoveError::IO(err) = err {
                    return Err(PartCleanupError::IO(err));
                }
            }
        }

        Ok(())
    }
}

type FileSystemAssembledFilePartsCleaner = AssembledFilePartsCleaner<
    FileSystemObjectStorageRemover,
    PostgresFileUploadRepository<MokaCache>,
    FileSystemObjectStorageCounter
>;

impl FileSystemAssembledFilePartsCleaner {
    fn get_with_settings(settings: &Settings) -> Self {
        Self::new(
            FileSystemObjectStorageRemover::new(),
            PostgresFileUploadRepository::get_with_settings(&settings),
            settings,
            FileSystemObjectStorageCounter::new()
        )
    }
}

#[derive(Debug)]
pub enum PartCleanupError {
    IO(String),
    NotFound(Uuid),
    NotChunked(Uuid),
}

impl From<FileUploadRepositoryError> for PartCleanupError {
    fn from(value: FileUploadRepositoryError) -> Self {
        match value {
            FileUploadRepositoryError::Pool(err) => Self::IO(err.to_string()),
            FileUploadRepositoryError::Db(err) => Self::IO(err.to_string()),
        }
    }
}

impl From<std::io::Error> for PartCleanupError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value.to_string())
    }
}

impl From<ObjectStorageRemoveError> for PartCleanupError {
    fn from(value: ObjectStorageRemoveError) -> Self {
        match value {
            ObjectStorageRemoveError::IO(err) => Self::IO(err.to_string()),
            ObjectStorageRemoveError::NotExists => Self::IO("file not exists".to_string()),
        }
    }
}

impl Display for PartCleanupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(err) => write!(f, "io error: {}", err),
            Self::NotFound(id) => write!(f, "file upload not found: {}", id),
            Self::NotChunked(id) => write!(f, "file upload is not chunked: {}", id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InMemoryPartCleanupJob<OBR, FR, OBC, L>
where
    OBR: ObjectStorageRemover + Clone,
    FR: FileUploadRepository + Clone,
    OBC: ObjectStorageCounter + Clone,
    L: ChunkedReaderLocker + Clone
{
    queue: Arc<Mutex<VecDeque<Uuid>>>,
    cleaner: AssembledFilePartsCleaner<OBR, FR, OBC>,
    locker: L,
    started: Arc<AtomicBool>,
}

impl<OBR, FR, OBC, L> InMemoryPartCleanupJob<OBR, FR, OBC, L>
where
    OBR: ObjectStorageRemover + Clone + Send + 'static,
    FR: FileUploadRepository + Clone + Send + 'static,
    OBC: ObjectStorageCounter + Clone + Send + 'static,
    L: ChunkedReaderLocker + Clone + Send + 'static
{
    pub fn new(cleaner: AssembledFilePartsCleaner<OBR, FR, OBC>, locker: L) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            cleaner,
            locker,
            started: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Try to execute next cleanup job, if it fails it will enqueue again the job
    pub fn try_next(&self) -> Result<(), String> {
        let id = self.queue.lock()
            .map_err(|err| format!("failed to acquire part cleanup queue lock: {}", err))?
            .pop_front();

        let id = match id {
            Some(id) => id,
            None => return Ok(()),
        };

        if self.locker.is_locked(&id)? {
            info!(upload = ?id, "parts are locked because maybe a download is in progress");
            info!(upload = ?id, "enqueuing again part cleanup job");

            self.queue.lock()
                .map_err(|err| format!("failed to acquire part cleanup queue lock: {}", err))?
                .push_back(id);

            return Ok(());
        }

        let result = self.cleaner.clean(&id);

        if let Err(err) = result {
            warn!(upload = ?id, error = ?err, "failed cleaning parts");
            info!(upload = ?id, "enqueuing again part cleanup job");

            self.queue.lock()
                .map_err(|err| format!("failed to acquire part cleanup queue lock: {}", err))?
                .push_back(id);
        }

        Ok(())
    }

    pub fn start(&self) {
        debug!("start part cleanup job");

        let started = self.started.load(Ordering::Relaxed);
        if started {
            debug!("part cleanup job already started");
            return;
        }

        let this = self.clone();
        std::thread::spawn(move || {
            loop {
                if let Err(err) = this.try_next() {
                    warn!(error = ?err, "failed to executing cleanup job");
                }

                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        });

        self.started.store(true, Ordering::Relaxed);
    }
}

impl<OBR, FR, OBC, L> PartCleanupJob for InMemoryPartCleanupJob<OBR, FR, OBC, L>
where
    OBR: ObjectStorageRemover + Clone,
    FR: FileUploadRepository + Clone,
    OBC: ObjectStorageCounter + Clone,
    L: ChunkedReaderLocker + Clone
{
    fn dispatch(&self, id: &Uuid) -> Result<(), String> {
        self.queue.lock()
            .map_err(|err| format!("failed to acquire part cleanup queue lock: {}", err))?
            .push_back(*id);

        Ok(())
    }
}

pub type FileSystemInMemoryPartCleanupJob = InMemoryPartCleanupJob<
    FileSystemObjectStorageRemover,
    PostgresFileUploadRepository<MokaCache>,
    FileSystemObjectStorageCounter,
    InMemoryChunkedReaderLocker
>;

static PART_CLEANUP_JOB_INSTANCE: OnceLock<FileSystemInMemoryPartCleanupJob> = OnceLock::new();

impl FileSystemInMemoryPartCleanupJob {
    pub fn initialize(settings: &Settings) {
        let instance = Self::new(
            FileSystemAssembledFilePartsCleaner::get_with_settings(&settings),
            InMemoryChunkedReaderLocker::get_instance(),
        );

        if let Err(_) = PART_CLEANUP_JOB_INSTANCE.set(instance.clone()) {
            warn!("FileSystemInMemoryPartCleanupJob is already initialized");
        } else {
            instance.start();
        }
    }

    /// Returns the current instance of the job, it is not initialized it will panic
    pub fn get_instance() -> Self {
        match PART_CLEANUP_JOB_INSTANCE.get() {
            Some(instance) => instance.clone(),
            None => panic!("Cleanup job instance must be initialized"),
        }
    }
}
