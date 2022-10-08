use std::path::{Path};

use anyhow::Result;
use crossbeam::sync::ShardedLock;
use serde::Deserialize;

use crate::yaml_utils::yaml_to_struct;

#[derive(Debug, Deserialize, Clone)]
pub struct Service {
    pub ip: String,
    pub port: String,
}

impl Default for Service {
    fn default() -> Self {
        Service {
            ip: "localhost".to_string(),
            port: "8080".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ProxyProperties {
    pub service: Service,
}

#[derive(Debug)]
struct ProxyConfig {
    props: ProxyProperties,
}

pub struct Configuration {
    proxy_config: ShardedLock<ProxyConfig>,
}

impl Configuration {
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let props = yaml_to_struct(&path)?;
        Ok(Configuration {
            proxy_config: ShardedLock::new(ProxyConfig { props }),
        })
    }

    pub fn service_config(&self) -> Service {
        self.proxy_config
            .read()
            .expect("proxy config read lock poisoned!")
            .props
            .service
            .clone()
    }
}

