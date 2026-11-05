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

use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, warn, info};
use uuid::Uuid;
use crate::cache::MokaCache;
use crate::config::Settings;
use crate::file::{FileUploadRepository, FileUploadRepositoryError, PostgresFileUploadRepository};
use crate::file::lock::{ChunkedReaderLocker, InMemoryChunkedReaderLocker};
use crate::storage::{FileSystemObjectStorageAppender, FileSystemObjectStorageReader, FileSystemObjectStorageRemover, ObjectStorageAppendError, ObjectStorageAppender, ObjectStorageReader, ObjectStorageRemoveError, ObjectStorageRemover};

pub struct FilePart;

impl FilePart {
    pub fn name(index: usize) -> String {
        format!("{}.part", index)
    }
}

pub struct FileSystemPendingFilePartsFinder;

impl FileSystemPendingFilePartsFinder {
    pub fn find(directory: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut parts = Vec::new();

        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let name = entry.file_name();
            let index = name.to_str()
                .and_then(|name| name.strip_suffix(".part"))
                .and_then(|index| index.parse::<usize>().ok());

            if let Some(index) = index {
                if name == FilePart::name(index).as_str() && entry.file_type()?.is_file() {
                    parts.push(entry.path());
                }
            }
        }

        Ok(parts)
    }
}

pub trait PartCleanupDispatcher {
    fn dispatch(&self, id: &Uuid) -> Result<(), String>;
}

#[derive(Clone)]
pub struct FileAssembler<FR, OBR, OBA, PCJ>
where
    FR: FileUploadRepository + Clone,
    OBR: ObjectStorageReader + Clone,
    OBA: ObjectStorageAppender + Clone,
    PCJ: PartCleanupDispatcher + Clone,
{
    settings: Settings,
    repository: FR,
    reader: OBR,
    appender: OBA,
    cleanup: PCJ,
}

impl<FR, OBR, OBA, PCJ> FileAssembler<FR, OBR, OBA, PCJ>
where
    FR: FileUploadRepository + Clone,
    OBR: ObjectStorageReader + Clone,
    OBA: ObjectStorageAppender + Clone,
    PCJ: PartCleanupDispatcher + Clone,
{
    pub fn new(settings: &Settings, repository: FR, reader: OBR, appender: OBA, cleanup_job: PCJ) -> Self {
        Self { settings: settings.clone(), repository, reader, appender, cleanup: cleanup_job }
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

        if let Err(err) = self.cleanup.dispatch(id) {
            warn!(upload = ?id, error = err, "failed to equeue file part cleanup job")
        }

        Ok(())
    }
}

impl FileAssembler<
    PostgresFileUploadRepository<MokaCache>,
    FileSystemObjectStorageReader,
    FileSystemObjectStorageAppender,
    FileSystemPartCleanupQueue
> {
    pub fn get_with_settings(settings: &Settings) -> Self {
        Self {
            settings: settings.clone(),
            repository: PostgresFileUploadRepository::get_with_settings(&settings),
            reader: FileSystemObjectStorageReader::new(),
            appender: FileSystemObjectStorageAppender::new(),
            cleanup: FileSystemPartCleanupQueue::get_instance().clone()
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
pub struct AssembledFilePartsCleaner<OBR, FR>
where
    OBR: ObjectStorageRemover + Clone,
    FR: FileUploadRepository + Clone,
{
    remover: OBR,
    repository: FR,
    settings: Settings,
}

impl<OBR, FR> AssembledFilePartsCleaner<OBR, FR>
where
    OBR: ObjectStorageRemover + Clone,
    FR: FileUploadRepository + Clone,
{
    pub fn new(remover: OBR, repository: FR, settings: &Settings) -> Self {
        Self {
            remover,
            repository,
            settings: settings.clone()
        }
    }

    pub fn clean(&self, id: &Uuid) -> Result<(), PartCleanupError> {
        let file = match self.repository.find_by_id(id)? {
            Some(file) => file,
            None => return Err(PartCleanupError::NotFound(id.clone()))
        };

        let content_path = file.content_directory(&self.settings)?;
        let parts = FileSystemPendingFilePartsFinder::find(&content_path)?;

        for part in parts {
            let path = part.to_string_lossy().to_string();

            if let Err(err) = self.remover.remove(path) {
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
    PostgresFileUploadRepository<MokaCache>
>;

impl FileSystemAssembledFilePartsCleaner {
    fn get_with_settings(settings: &Settings) -> Self {
        Self::new(
            FileSystemObjectStorageRemover::new(),
            PostgresFileUploadRepository::get_with_settings(&settings),
            settings
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

pub trait PartCleanupQueue: Clone {
    fn push(&self, id: Uuid) -> Result<(), String>;
    fn poll(&self) -> Result<PartCleanupDelivery, String>;
}

pub struct PartCleanupDelivery {
    pub id: Uuid,
    files: std::sync::mpsc::Sender<Uuid>,
}

impl PartCleanupDelivery {
    pub fn complete(self) -> Result<(), String> {
        self.files.send(self.id).map_err(|err| err.to_string())
    }
}

static FILE_SYSTEM_PART_CLEANUP_QUEUE_INSTANCE: OnceLock<FileSystemPartCleanupQueue> = OnceLock::new();

#[derive(Clone)]
pub struct FileSystemPartCleanupQueue {
    sender: std::sync::mpsc::Sender<Uuid>,
    receiver: Arc<Mutex<std::sync::mpsc::Receiver<Uuid>>>,
    files: std::sync::mpsc::Sender<Uuid>,
    directory: PathBuf,
}

impl FileSystemPartCleanupQueue {
    pub fn new(directory: &Path) -> Self {
        if let Err(err) = std::fs::create_dir_all(directory) {
            warn!(error = %err, "failed to create queue directory");
        }

        let (sender, receiver) = std::sync::mpsc::channel();
        let (files, actions) = std::sync::mpsc::channel();

        if let Err(err) = Self::restore_pending(directory, &sender) {
            warn!(error = %err, "failed to restore pending queue");
        }

        let directory = directory.to_path_buf();
        let cleanup_directory = directory.clone();
        std::thread::spawn(move || Self::remove_completed(&cleanup_directory, actions));

        Self { sender, receiver: Arc::new(Mutex::new(receiver)), files, directory }
    }

    pub fn initialize_with_settings(settings: &Settings) -> Self {
        FILE_SYSTEM_PART_CLEANUP_QUEUE_INSTANCE.get_or_init(|| {
            let path = Path::new(&settings.server.state_directory);
            Self::new(&path.join("part_cleanup_queue"))
        }).clone()
    }

    /// Get the queue current instance, it must be initialized, if not it panic
    pub fn get_instance<'a>() -> &'a Self {
        FILE_SYSTEM_PART_CLEANUP_QUEUE_INSTANCE.get().unwrap()
    }

    fn restore_pending(directory: &Path, sender: &std::sync::mpsc::Sender<Uuid>) -> Result<(), String> {
        for entry in std::fs::read_dir(directory).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;

            if let Some(id) = entry.file_name().to_str().and_then(|name| Uuid::parse_str(name).ok()) {
                sender.send(id).map_err(|err| err.to_string())?;
            }
        }

        Ok(())
    }

    fn remove_completed(directory: &Path, completed: std::sync::mpsc::Receiver<Uuid>) {
        while let Ok(id) = completed.recv() {
            if let Err(err) = std::fs::remove_file(directory.join(id.to_string())) {
                if err.kind() != std::io::ErrorKind::NotFound {
                    warn!(error = %err, upload = %id, "failed removing completed part cleanup message");
                }
            }
        }
    }
}

impl PartCleanupQueue for FileSystemPartCleanupQueue {
    fn push(&self, id: Uuid) -> Result<(), String> {
        if let Err(err) = std::fs::write(self.directory.join(id.to_string()), id.as_bytes()) {
            warn!(error = %err, upload = %id, "failed persisting part cleanup message");
        }

        self.sender.send(id).map_err(|err| err.to_string())?;

        Ok(())
    }

    fn poll(&self) -> Result<PartCleanupDelivery, String> {
        let id = self.receiver.lock()
            .map_err(|err| err.to_string())?
            .recv()
            .map_err(|err| err.to_string())?;

        Ok(PartCleanupDelivery { id, files: self.files.clone() })
    }
}

#[derive(Clone)]
pub struct PartCleanupWorker<OBR, FR, L, Q>
where
    OBR: ObjectStorageRemover + Clone,
    FR: FileUploadRepository + Clone,
    L: ChunkedReaderLocker + Clone,
    Q: PartCleanupQueue,
{
    queue: Q,
    cleaner: AssembledFilePartsCleaner<OBR, FR>,
    locker: L,
    started: Arc<AtomicBool>,
}

impl<OBR, FR, L, Q> PartCleanupWorker<OBR, FR, L, Q>
where
    OBR: ObjectStorageRemover + Clone + Send + 'static,
    FR: FileUploadRepository + Clone + Send + 'static,
    L: ChunkedReaderLocker + Clone + Send + 'static,
    Q: PartCleanupQueue + Send + 'static,
{
    pub fn new(cleaner: AssembledFilePartsCleaner<OBR, FR>, locker: L, queue: Q) -> Self {
        Self { queue, cleaner, locker, started: Arc::new(AtomicBool::new(false)) }
    }

    pub fn try_next(&self) -> Result<(), String> {
        let delivery = self.queue.poll()?;
        let id = delivery.id;
        let locked = match self.locker.is_locked(&id) {
            Ok(locked) => locked,
            Err(err) => {
                self.queue.push(id)?;
                return Err(err);
            }
        };

        if locked {
            self.queue.push(id)?;
            std::thread::sleep(std::time::Duration::from_millis(100));
            return Ok(());
        }

        if let Err(err) = self.cleaner.clean(&id) {
            warn!(upload = ?id, error = ?err, "failed cleaning parts");
            self.queue.push(id)?;
            std::thread::sleep(std::time::Duration::from_millis(100));
        } else {
            delivery.complete()?;
        }

        Ok(())
    }

    pub fn start(&self) {
        if self.started.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
            return;
        }

        let this = self.clone();
        std::thread::spawn(move || loop {
            if let Err(err) = this.try_next() {
                warn!(error = ?err, "failed executing cleanup job");
            }
        });
    }
}

impl PartCleanupDispatcher for FileSystemPartCleanupQueue {
    fn dispatch(&self, id: &Uuid) -> Result<(), String> {
        self.push(id.clone()).map_err(|err| err.to_string())
    }
}

pub type FileSystemPartCleanupWorker = PartCleanupWorker<
    FileSystemObjectStorageRemover,
    PostgresFileUploadRepository<MokaCache>,
    InMemoryChunkedReaderLocker,
    FileSystemPartCleanupQueue
>;

impl FileSystemPartCleanupWorker {
    /// initializes the queue and start consuming it
    pub fn start_consuming(settings: &Settings) {
        let instance = Self::new(
            FileSystemAssembledFilePartsCleaner::get_with_settings(&settings),
            InMemoryChunkedReaderLocker::get_instance(),
            FileSystemPartCleanupQueue::initialize_with_settings(&settings)
        );

        instance.start();
    }
}

#[cfg(test)]
mod cleanup_tests {
    use super::*;
    use crate::file::{FileUpload, InMemoryFileUploadRepository};
    use std::collections::HashMap;

    #[test]
    fn removes_remaining_parts_with_gaps_in_their_indices() {
        let id = Uuid::new_v4();
        let root = std::env::temp_dir().join(format!("mikupush-parts-{id}"));
        let mut settings = Settings::default();
        settings.upload.directory = root.to_string_lossy().into_owned();
        let upload = FileUpload {
            id,
            name: "content.bin".to_string(),
            mime_type: "application/octet-stream".to_string(),
            size: 6,
            uploaded_at: chrono::Utc::now().naive_utc(),
            chunked: false,
            expires_at: None,
        };
        let directory = upload.content_directory(&settings).unwrap();
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("4.part"), b"abc").unwrap();
        std::fs::write(directory.join("9.part"), b"def").unwrap();
        std::fs::write(directory.join("content.bin"), b"abcdef").unwrap();
        std::fs::write(directory.join("notes.part"), b"notes").unwrap();
        let repository = InMemoryFileUploadRepository::new(HashMap::from([(id, upload)]));
        let cleaner = AssembledFilePartsCleaner::new(FileSystemObjectStorageRemover::new(), repository, &settings);

        cleaner.clean(&id).unwrap();
        cleaner.clean(&id).unwrap();

        assert!(FileSystemPendingFilePartsFinder::find(&directory).unwrap().is_empty());
        assert_eq!(std::fs::read(directory.join("content.bin")).unwrap(), b"abcdef");
        assert!(directory.join("notes.part").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
