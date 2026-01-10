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


mod settings;
mod logging;
mod yaml;
mod env;

pub use logging::*;
pub use settings::*;

use std::path::PathBuf;


pub fn user_config_path() -> PathBuf {
    #[cfg(target_os = "linux")]
    let paths: Vec<PathBuf> = vec![
        PathBuf::from("config.yaml"),
        PathBuf::from(format!("{}/.io.mikupush.server/config.yaml", env!("HOME"))),
        PathBuf::from(format!("{}/.config/io.mikupush.server/config.yaml", env!("HOME"))),
        PathBuf::from("/etc/io.mikupush.server/config.yaml"),
    ];

    #[cfg(target_os = "windows")]
    let paths: Vec<PathBuf> = vec![
        PathBuf::from("config.yaml"),
        PathBuf::from(format!("{}\\io.mikupush.server\\config.yaml", env!("LOCALAPPDATA"))),
    ];

    #[cfg(target_os = "macos")]
    let paths: Vec<PathBuf> = vec![
        PathBuf::from("config.yaml"),
        PathBuf::from(format!("{}/.io.mikupush.server/config.yaml", env!("HOME"))),
        PathBuf::from(format!("{}/.config/io.mikupush.server/config.yaml", env!("HOME"))),
        PathBuf::from(format!("{}/Library/Application Support/io.mikupush.server/config.yaml", env!("HOME"))),
    ];

    for path in paths {
        println!("attempting to find configuration file: {}", path.display());

        if path.exists() {
            println!("configuration file found: {}", path.display());
            return path;
        }
    }

    PathBuf::from("config.yaml")
}
