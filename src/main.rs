extern crate rpgd;
extern crate ws;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ws::listen;

use rpgd::server::client::Client;
use rpgd::server::connection::Connection;

fn main() {
    println!("Hello, rpgd!");
    let registry = Rc::new(RefCell::new(HashMap::new()));

    listen("127.0.0.1:7474", move |connexion| Connection {
        client: Client {
            sender: connexion,
            nick: "".to_string(),
            ready: false,
        },
        registry: registry.clone(),
    }).unwrap()
}
