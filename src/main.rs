extern crate ws;

use std::collections::HashMap;
use ws::util::Token;
use ws::{listen, Handler, Handshake, Message, Result, Sender};

fn main() {
    println!("Hello, rpgd!");

    struct Server {
        clients: HashMap<Token, Sender>,
    }

    impl Handler for Server {
        fn on_open(&mut self, shake: Handshake) -> Result<()> {
            println!("Server got connexion {:?}", shake);
            Ok(())
        }

        fn on_message(&mut self, msg: Message) -> Result<()> {
            println!("Server got message {:?}", msg);
            for (token, client) in &self.clients {
                let cmsg = msg.clone();
                client.send(cmsg);
            }
            Ok(())
        }
    }

    let mut clients = HashMap::new();

    listen("127.0.0.1:7474", move |connexion| {
        println!("new connexion");

        {
            clients.insert(connexion.token(), connexion)
        };
        {
            Server { clients: clients.clone() }
        }
    }).unwrap()
}
