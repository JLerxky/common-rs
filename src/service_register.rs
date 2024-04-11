use color_eyre::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServiceRegisterConfig {
    pub url: String,
    pub tags: Vec<String>,
    pub ttl: i64,
}

impl Default for ServiceRegisterConfig {
    fn default() -> Self {
        Self {
            tags: Default::default(),
            ttl: 60,
            url: Default::default(),
        }
    }
}

pub trait ServiceRegister {
    fn keep_service_register(
        &self,
        service_name: &str,
        config: ServiceRegisterConfig,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}
