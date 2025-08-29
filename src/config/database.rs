use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DataBase {
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    user: Option<String>,
    #[serde(default)]
    password: Option<String>,
}

impl DataBase {
    pub fn url(&self) -> String {
        std::env::var("MIKU_PUSH_DATABASE_URL").ok()
            .or_else(|| self.url.clone())
            .or_else(|| Some("postgres://localhost:5432/postgres".to_string()))
            .unwrap()
    }

    pub fn user(&self) -> String {
        std::env::var("MIKU_PUSH_DATABASE_USER").ok()
            .or_else(|| self.user.clone())
            .or_else(|| Some("postgres".to_string()))
            .unwrap()
    }

    pub fn password(&self) -> String {
        std::env::var("MIKU_PUSH_DATABASE_PASSWORD").ok()
            .or_else(|| self.password.clone())
            .or_else(|| Some("postgres".to_string()))
            .unwrap()
    }
}

impl Default for DataBase {
    fn default() -> Self {
        DataBase {
            url: None,
            user: None,
            password: None,
        }
    }
}
