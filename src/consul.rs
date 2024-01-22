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

use std::collections::HashMap;

use color_eyre::eyre::{eyre, Ok, Result};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConsulConfig {
    pub consul_addr: String,
    pub tags: Vec<String>,
    pub service_name: String,
    pub service_id: String,
    pub service_address: String,
    pub service_port: u16,
    pub check_http_path: String,
    pub check_interval: String,
    pub check_timeout: String,
    pub check_deregister_critical_service_after: String,
}

async fn put_service_register(config: &ConsulConfig) -> Result<()> {
    let uri = format!("{}/v1/agent/service/register", config.consul_addr);

    let rsp = reqwest::Client::default().put(uri).body(json!({
            "ID": config.service_id,
            "Name": config.service_name,
            "Port": config.service_port,
            "Tags": config.tags,
            "Address": config.service_address,
            "Check": {
                "Name": config.service_name.clone() + "_check",
                "DeregisterCriticalServiceAfter": config.check_deregister_critical_service_after,
                "HTTP": format!("http://{}:{}{}", config.service_address, config.service_port, config.check_http_path),
                "Interval": config.check_interval,
                "Timeout": config.check_timeout,
            }
        }).to_string()).send().await
        .map_err(|e| eyre!("register to consul failed: {e}"))?;

    if rsp.status() != StatusCode::OK {
        Err(eyre!("register to consul failed: {rsp:?}"))
    } else {
        debug!("register to consul: {:?}", config);
        Ok(())
    }
}

pub async fn service_register(config: &ConsulConfig) -> Result<()> {
    let registered_services = get_registered_services(config).await?;
    if registered_services.contains_key(&config.service_name) {
        Ok(())
    } else {
        put_service_register(config).await
    }
}

pub async fn get_registered_services(
    config: &ConsulConfig,
) -> Result<HashMap<String, Vec<String>>> {
    let uri = format!("{}/v1/catalog/services", config.consul_addr);

    let rsp = reqwest::Client::default()
        .get(uri)
        .send()
        .await
        .map_err(|e| eyre!("consul get all_registered_services failed: {e}"))?;

    if rsp.status() != StatusCode::OK {
        Err(eyre!("consul get all_registered_services failed: {rsp:?}"))
    } else {
        let service_tags_by_name = serde_json::from_slice::<HashMap<String, Vec<String>>>(
            &rsp.bytes()
                .await
                .map_err(|e| eyre!("all_registered_services read bytes failed: {e}"))?,
        )
        .map_err(|e| eyre!("all_registered_services decode service_tags_by_name failed: {e}"))?;
        Ok(service_tags_by_name)
    }
}

pub async fn read_raw_key(consul_addr: &str, key: &str) -> Result<String> {
    let uri = format!("{}/v1/kv/{}?raw", consul_addr, key);

    let rsp = reqwest::Client::default()
        .get(uri)
        .send()
        .await
        .map_err(|e| eyre!("read key from consul failed: {e}"))?;

    if rsp.status().is_success() {
        let raw_value = std::str::from_utf8(
            &rsp.bytes()
                .await
                .map_err(|e| eyre!("read key from consul failed: {e}"))?,
        )
        .map_err(|e| eyre!("raw_value from_utf8 failed: {e}"))?
        .to_string();
        debug!("get raw_value from consul: {}", raw_value);
        Ok(raw_value)
    } else {
        Err(eyre!("read key from consul failed: {rsp:?}"))
    }
}
