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

use tracing::info;

pub struct ElapsedTimeTracing {
    label: String,
    start_time: std::time::Instant,
}

impl ElapsedTimeTracing {
    pub fn new(label: &str) -> Self {
        Self { 
            label: label.to_string(), 
            start_time: std::time::Instant::now() 
        }
    }

    pub fn trace(&self) {
        let elapsed = self.start_time.elapsed();
        let elapsed_ms = elapsed.as_millis();
        info!(time_ms = elapsed_ms, label = self.label, "{} took {} ms", self.label, elapsed_ms);
    }
}