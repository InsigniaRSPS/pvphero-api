use config::{ConfigError, Config, File};

#[derive(Debug, Deserialize)]
pub struct Server {
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Redis {
    pub port: u16,
    pub url: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub redis: Redis,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("config/default"))?;
        s.try_into()
    }
}