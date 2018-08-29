#[macro_use]
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate wardice;
extern crate ws;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ws::listen;

mod server;

use server::client::Client;
use server::connection::Connection;

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
