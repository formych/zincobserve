// Copyright 2022 Zinc Labs Inc. and Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

pub mod alert;
pub mod common;
pub mod dashboards;
pub mod functions;
pub mod http;
pub mod ingestion;
pub mod organization;
pub mod prom;
pub mod search;
pub mod service;
pub mod sql;
pub mod stream;
pub mod telemetry;
pub mod traces;
pub mod user;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum StreamType {
    #[default]
    #[serde(rename = "logs")]
    Logs,
    #[serde(rename = "metrics")]
    Metrics,
    #[serde(rename = "traces")]
    Traces,
    #[serde(rename = "metadata")]
    Metadata,
    #[serde(rename = "file_list")]
    Filelist,
}

impl From<&str> for StreamType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "logs" => StreamType::Logs,
            "metrics" => StreamType::Metrics,
            "traces" => StreamType::Traces,
            "metadata" => StreamType::Metadata,
            "file_list" => StreamType::Filelist,
            _ => StreamType::Logs,
        }
    }
}

impl fmt::Display for StreamType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StreamType::Logs => write!(f, "logs"),
            StreamType::Metrics => write!(f, "metrics"),
            StreamType::Traces => write!(f, "traces"),
            StreamType::Metadata => write!(f, "metadata"),
            StreamType::Filelist => write!(f, "file_list"),
        }
    }
}
