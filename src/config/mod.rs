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

mod database;
mod server;
mod settings;
mod upload;
mod logging;

pub use database::*;
pub use logging::*;
pub use server::*;
pub use settings::*;
pub use upload::*;

use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, Mutex, Once};
use tracing::debug;

fn load_dotenv() -> HashMap<String, String> {
    let mut env_files: VecDeque<&str> = VecDeque::new();
    env_files.push_back(".env");

    #[cfg(test)]
    env_files.push_back(".env.test");

    for env_file in env_files {
        debug!("loading dotenv file: {}", env_file);

        let dotenv_variables = dotenvy::from_filename_iter(env_file);
        if let Err(err) = dotenv_variables {
            debug!("failed to load dotenv file {}: {}", env_file, err);
            continue;
        }

        debug!("dotenv file {} loaded!", env_file);
        let mut variables = HashMap::new();
        for item in dotenv_variables.unwrap() {
            if let Ok((key, value)) = item {
                variables.insert(key, value);
            }
        }

        return variables
    }

    HashMap::new()
}

static DOTENV_LOADED: Once = Once::new();
static DOTENV_VARS: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

#[cfg(test)]
static TEST_ENV: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn env(name: &str) -> Option<String> {
    #[cfg(test)]
    {
        let test_env = TEST_ENV.lock().unwrap();
        if let Some(test_value) = test_env.get(name) {
            println!("using {}={} test variable", name, test_value);
            return Some(test_value.clone());
        }
    }

    DOTENV_LOADED.call_once(|| {
        let dotenv_vars = load_dotenv();
        *DOTENV_VARS.lock().unwrap() = dotenv_vars;
    });

    let dotenv_vars = DOTENV_VARS.lock().unwrap();
    if let Some(value) = dotenv_vars.get(name) {
        return Some(value.to_string());
    }

    if let Some(value) = std::env::var(name).ok() {
        return Some(value.to_string());
    }

    None
}

#[cfg(test)]
pub mod tests {
    use crate::config::{TEST_ENV, UPLOAD_MAX_SIZE_UNLIMITED};

    pub fn set_test_env(name: &str, value: &str) {
        let mut test_env = TEST_ENV.lock().unwrap();
        test_env.insert(name.to_string(), value.to_string());
    }

    pub fn setup_test_env() {
        set_test_env("MIKU_PUSH_UPLOAD_MAX_SIZE", UPLOAD_MAX_SIZE_UNLIMITED);
        set_test_env("MIKU_PUSH_UPLOAD_DIRECTORY", "data/tests")
    }
}