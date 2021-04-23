#[derive(Debug, Clone)]
pub struct Server {
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct Redis {
    pub port: u16,
    pub url: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub server: Server,
    pub redis: Redis,
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            server: Server {
                port: env!("API_SERVER_PORT").parse().expect("Unable to parse server port")
            },
            redis: Redis {
                port: env!("API_SERVER_REDIS_PORT").parse().expect("Unable to parse redis port"),
                url: env!("API_SERVER_REDIS_URL").parse().expect("Unable to parse redis url"),
                password: env!("API_SERVER_REDIS_PASSWORD").parse().expect("Unable to parse redis password"),
            },
        }
    }
}