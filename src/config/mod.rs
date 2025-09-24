/// Copyright 2025 Miku Push! Team
///
/// Licensed under the Apache License, Version 2.0 (the "License");
/// you may not use this file except in compliance with the License.
/// You may obtain a copy of the License at
///
///     http://www.apache.org/licenses/LICENSE-2.0
///
/// Unless required by applicable law or agreed to in writing, software
/// distributed under the License is distributed on an "AS IS" BASIS,
/// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
/// See the License for the specific language governing permissions and
/// limitations under the License.

mod database;
mod server;
mod settings;
mod upload;
mod logging;

pub use database::*;
pub use server::*;
pub use settings::*;
pub use upload::*;
pub use logging::*;

use tracing::debug;
use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, Mutex, Once};

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
