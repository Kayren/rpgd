extern crate dotenv;
extern crate log;
extern crate rpgd;
extern crate simple_logger;
extern crate ws;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ws::listen;

use rpgd::config;
use rpgd::server::client::Client;
use rpgd::server::connection::Connection;

fn main() {
    dotenv::dotenv().expect("No .env file found");
    let config = config::init();

    simple_logger::init_with_level(log::Level::Info).unwrap();

    let registry = Rc::new(RefCell::new(HashMap::new()));

    listen(format!("{}:{}", config.url, config.port), move |connexion| Connection {
        client: Client {
            sender: connexion,
            nick: "".to_string(),
            ready: false,
        },
        registry: registry.clone(),
    }).unwrap()
}
