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

use anyhow::{anyhow, Result};
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

pub async fn service_register(config: &ConsulConfig) -> Result<()> {
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
        .map_err(|e| anyhow!("register to consul failed: {e}"))?;

    if rsp.status() != StatusCode::OK {
        Err(anyhow!("register to consul failed: {rsp:?}"))
    } else {
        Ok(())
    }
}
