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

use crate::config::Settings;
use crate::file::{FileUploadRepository, PostgresFileUploadRepository};
use crate::storage::ObjectStorageRemover;
use crate::file::FileDeleter;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info};

pub fn start_cleanup_expired_files(settings: Settings) {
    if settings.upload.expires_in_seconds.is_none() {
        return;
    }

    info!("launching file cleanup job every {} seconds", settings.upload.expiration_cleanup_interval_seconds);

    thread::spawn(move || {
        debug!("expired file cleanup job started");

        loop {
            let repository = PostgresFileUploadRepository::get_with_settings(&settings);
            let deleter = FileDeleter::get_with_settings(&settings);

            cleanup_expired_files(repository, deleter);
            thread::sleep(Duration::from_secs(settings.upload.expiration_cleanup_interval_seconds));
        }
    });
}

pub fn cleanup_expired_files<FR, OSR>(repository: FR, deleter: FileDeleter<FR, OSR>)
where
    FR: FileUploadRepository + Clone,
    OSR: ObjectStorageRemover + Clone,
{
    info!("starting cleanup of expired files");
    let expired_files = match repository.find_expired() {
        Ok(expired_files) => expired_files,
        Err(e) => {
            error!("failed to fetch expired files: {:?}", e);
            return;
        }
    };

    if expired_files.is_empty() {
        info!("no expired files found");
        return;
    }

    info!("found {} expired files to delete", expired_files.len());
    for file in expired_files {
        match deleter.delete(file.id) {
            Ok(_) => debug!("deleted expired file: {}", file.id),
            Err(e) => error!("failed to delete expired file {}: {:?}", file.id, e),
        }
    }

    info!("finished cleanup of expired files");
}
