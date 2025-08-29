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
        std::env::var("MIKU_PUSH_SERVER_HOST").ok()
            .or_else(|| self.host.clone())
            .or_else(|| Some("0.0.0.0".to_string()))
            .unwrap()
    }

    pub fn port(&self) -> u16 {
        std::env::var("MIKU_PUSH_SERVER_PORT").ok()
            .map(|s| s.parse::<u16>().expect("Server port must be a number"))
            .or_else(|| self.port.clone())
            .or_else(|| Some(8080))
            .unwrap()
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
