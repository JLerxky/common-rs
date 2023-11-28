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

use std::{fmt::Debug, path::Path, sync::Arc};

use async_trait::async_trait;
use color_eyre::eyre::{eyre, Result};
use config::{AsyncSource, Config, ConfigError, FileFormat, Map};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use serde::Deserialize;
use tracing::{error, info};

pub fn file_config<T: for<'a> Deserialize<'a>>(path: &str) -> Result<T> {
    let settings = Config::builder()
        .add_source(config::File::with_name(path))
        .build()
        .map_err(|e| eyre!("load file config failed: {}", e))?;

    settings
        .try_deserialize::<T>()
        .map_err(|e| eyre!("deserialize config failed: {}", e))
}

pub async fn async_config(uri: &str) -> Result<Config> {
    Config::builder()
        .add_async_source(HttpSource {
            uri: uri.into(),
            format: FileFormat::Json,
        })
        .build()
        .await
        .map_err(|e| eyre!("load async config failed: {}", e))
}

#[derive(Debug)]
pub struct HttpSource<F: config::Format> {
    uri: String,
    format: F,
}

#[async_trait]
impl<F: config::Format + Send + Sync + Debug> AsyncSource for HttpSource<F> {
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

pub fn config_hot_reload<T: for<'a> Deserialize<'a> + Sync + Send + 'static>(
    config: Arc<RwLock<T>>,
    config_path: String,
) -> Result<()> {
    let config_path_clone = config_path.clone();
    // reload config
    let mut watcher = RecommendedWatcher::new(
        move |result: Result<Event, notify::Error>| {
            let event = result.unwrap();

            if event.kind.is_modify() {
                match Figment::new()
                    .join(Toml::file(&config_path_clone))
                    .extract()
                {
                    Ok(new_config) => {
                        info!("reloading config");
                        *config.write() = new_config;
                    }
                    Err(error) => error!("Error reloading config: {:?}", error),
                }
            }
        },
        notify::Config::default(),
    )?;
    watcher.watch(Path::new(&config_path), RecursiveMode::Recursive)?;
    Ok(())
}
