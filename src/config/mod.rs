use std::env;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub url: String,
    pub port: String,
}

pub fn init() -> Config {
    let mut config: Config = Config {
        url: "".to_string(),
        port: "".to_string(),
    };

    // check presence of URL
    config.url = env::var("RPGD_URL").expect("RPGD_URL env var not found");

    // check presence of PORT
    config.port = env::var("RPGD_PORT").expect("RPGD_PORT env var not found");

    config
}
