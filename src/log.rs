// Copyright Rivtower Technologies LLC.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use chrono::{Local, Offset};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use time::{format_description::well_known, UtcOffset};
use tracing_subscriber::{fmt::format, fmt::time::OffsetTime, prelude::*, EnvFilter};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    max_level: String,
    filter: String,
    rolling_file_path: Option<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            max_level: "info".to_owned(),
            filter: "info".to_owned(),
            rolling_file_path: Default::default(),
        }
    }
}

pub fn init_tracing(name: &str, log_config: &LogConfig) -> Result<()> {
    // log
    let mut logfile = None;
    let mut stdout = None;
    if let Some(rolling_file_path) = &log_config.rolling_file_path {
        // logfile
        logfile = Some(tracing_appender::rolling::daily(rolling_file_path, name));
    } else {
        // stdout
        stdout = Some(
            std::io::stdout
                .with_max_level(tracing::Level::from_str(&log_config.max_level).unwrap()),
        );
    }

    // set timer
    let local_offset_sec = Local::now().offset().fix().local_minus_utc();
    let utc_offset = UtcOffset::from_whole_seconds(local_offset_sec)
        .unwrap_or(UtcOffset::from_hms(8, 0, 0).unwrap());
    let timer = OffsetTime::new(utc_offset, well_known::Rfc3339);

    if let Some(stdout) = stdout {
        tracing_subscriber::registry()
            .with(EnvFilter::new(&log_config.filter))
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(format().compact())
                    .with_ansi(false)
                    .with_timer(timer)
                    .with_writer(stdout),
            )
            .try_init()?;
    } else {
        tracing_subscriber::registry()
            .with(EnvFilter::new(&log_config.filter))
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(format().compact())
                    .with_ansi(false)
                    .with_timer(timer)
                    .with_writer(logfile.unwrap()),
            )
            .try_init()?;
    }

    Ok(())
}
