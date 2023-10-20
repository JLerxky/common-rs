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

use std::fmt::Debug;

use anyhow::{anyhow, Result};
use axum::async_trait;
use config::{AsyncSource, Config, ConfigError, FileFormat, Format, Map};
use serde::Deserialize;

pub fn file_config<T: for<'a> Deserialize<'a>>(path: &str) -> Result<T> {
    let settings = Config::builder()
        .add_source(config::File::with_name(path))
        .build()
        .map_err(|e| anyhow!("load file config failed: {}", e))?;

    settings
        .try_deserialize::<T>()
        .map_err(|e| anyhow!("deserialize config failed: {}", e))
}

pub async fn async_config(uri: &str) -> Result<Config> {
    Config::builder()
        .add_async_source(HttpSource {
            uri: uri.into(),
            format: FileFormat::Json,
        })
        .build()
        .await
        .map_err(|e| anyhow!("load async config failed: {}", e))
}

#[derive(Debug)]
pub struct HttpSource<F: Format> {
    uri: String,
    format: F,
}

#[async_trait]
impl<F: Format + Send + Sync + Debug> AsyncSource for HttpSource<F> {
    async fn collect(&self) -> Result<Map<String, config::Value>, ConfigError> {
        reqwest::get(&self.uri)
            .await
            .map_err(|e| ConfigError::Foreign(Box::new(e)))?
            .text()
            .await
            .map_err(|e| ConfigError::Foreign(Box::new(e)))
            .and_then(|text| {
                self.format
                    .parse(Some(&self.uri), &text)
                    .map_err(ConfigError::Foreign)
            })
    }
}
