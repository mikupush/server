use log::debug;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    #[serde(default)]
    host: Option<String>,
    #[serde(default)]
    port: Option<u16>,
}

impl Server {
    pub fn host(&self) -> String {
        let value = std::env::var("MIKU_PUSH_SERVER_HOST").ok();
        if let Some(value) = value {
            debug!("using env variable MIKU_PUSH_SERVER_HOST: {}", value);
            return value;
        }

        let value = self.host.clone();
        if let Some(value) = value {
            debug!("using server.host configuration: {}", value);
            return value;
        }

        let value = "0.0.0.0".to_string();
        debug!("using server.host default value: {}", value);
        value
    }

    pub fn port(&self) -> u16 {
        let value = std::env::var("MIKU_PUSH_SERVER_PORT").ok();
        if let Some(value) = value {
            debug!("using env variable MIKU_PUSH_SERVER_PORT: {}", value);
            return value.parse().expect("Server port must be a number");
        }

        let value = self.port.clone();
        if let Some(value) = value {
            debug!("using server.port configuration: {}", value);
            return value;
        }

        let value = 8080;
        debug!("using server.port default value: {}", value);
        value
    }
}

impl Default for Server {
    fn default() -> Self {
        Server {
            host: None,
            port: None,
        }
    }
}
