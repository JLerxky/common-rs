use color_eyre::{eyre::eyre, Result};
pub use redis::*;

use serde::{Deserialize, Serialize};

use tracing::{debug, error};

use crate::service_register::{ServiceRegister, ServiceRegisterConfig};

cfg_if::cfg_if! {
    if #[cfg(feature = "redis-cluster")] {
        pub use redis::{cluster::ClusterClient as RedisClient, cluster_async::ClusterConnection as RedisConnection};
    } else {
        pub use redis::{aio::MultiplexedConnection as RedisConnection, Client as RedisClient};
    }
}

#[derive(Clone)]
pub struct Redis {
    client: RedisClient,
    connection: RedisConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RedisConfig {
    pub endpoints: Vec<String>,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            endpoints: vec!["redis://127.0.0.1/".to_owned()],
        }
    }
}

impl Redis {
    pub async fn new(config: &RedisConfig) -> Result<Self> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "redis-cluster")] {
                let client = RedisClient::new(config.endpoints.clone())
                    .map_err(|e| eyre!("redis connect failed: {e}"))?;

                let connection = client
                    .get_async_connection()
                    .await
                    .map_err(|e| eyre!("redis connect failed: {e}"))?;
            } else {
                let client = Client::open(config.endpoints[0].clone())
                    .map_err(|e| eyre!("redis connect failed: {e}"))?;

                let connection = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| eyre!("redis connect failed: {e}"))?;
            }
        }
        Ok(Self { client, connection })
    }

    pub fn client(&self) -> RedisClient {
        self.client.to_owned()
    }

    pub fn conn(&self) -> RedisConnection {
        self.connection.to_owned()
    }

    pub async fn keep_alive(&mut self) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(not(feature = "redis-cluster"))] {
                if !self.client.check_connection() {
                    if let Ok(new_conn) = self
                        .client
                        .get_multiplexed_async_connection()
                        .await
                        .map_err(|e| eyre!("redis connect failed: {e}"))
                    {
                        self.connection = new_conn;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn service_register(
        &self,
        service_name: &str,
        config: ServiceRegisterConfig,
    ) -> Result<()> {
        self.keep_service_register(service_name, config).await
    }
}

impl ServiceRegister for Redis {
    async fn keep_service_register(
        &self,
        service_name: &str,
        config: ServiceRegisterConfig,
    ) -> Result<()> {
        debug!("keep_service_register: {config:#?}");
        let mut keep_alive_interval =
            tokio::time::interval(tokio::time::Duration::from_secs((config.ttl / 2) as u64));

        let redis = self.clone();
        let service_name = service_name.to_owned();
        tokio::spawn(async move {
            loop {
                keep_alive_interval.tick().await;
                let tags = config.tags.clone();
                let service_name = service_name.clone();

                match redis
                    .conn()
                    .set_ex(
                        format!(
                            "traefik/http/services/{}/loadbalancer/servers/{}/url",
                            service_name, service_name
                        ),
                        config.url.clone(),
                        config.ttl as u64,
                    )
                    .await
                {
                    Ok(()) => {}
                    Err(e) => error!("keep_service_register failed: {:?}", e),
                }
                match redis
                    .conn()
                    .set_ex(
                        format!("traefik/http/routers/{}/service", service_name),
                        service_name,
                        config.ttl as u64,
                    )
                    .await
                {
                    Ok(()) => {}
                    Err(e) => error!("keep_service_register failed: {:?}", e),
                }
                for tag in tags {
                    let (key, value) = tag.split_once('=').unwrap_or_default();
                    match redis.conn().set_ex(key, value, config.ttl as u64).await {
                        Ok(()) => {}
                        Err(e) => error!("keep_service_register failed: {:?}", e),
                    }
                }
            }
        });
        Ok(())
    }
}
