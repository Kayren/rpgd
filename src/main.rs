extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate wardice;
extern crate ws;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ws::util::Token;
use ws::{listen, CloseCode, Handler, Handshake, Message, Result, Sender};

#[derive(Deserialize, Debug)]
#[serde(tag = "cmd")]
/// List o f message who send to the serve
/// ###
/// Example
/// Set_Name {name: "test"} => {"cmd":"Set_Name","name":"test"}
/// Roll_dice { dices: [
///     Characteristic,
///     Characteristic,
///     Characteristic,
///     Challenge,
///     Challenge]
///  }
/// => {"cmd":"Roll_dice", "dices": ["Characteristic","Characteristic","Characteristic","Challenge","Challenge"]}
enum Ws_Message_In {
    Set_Name { name: String },
    Roll_dice { dices: Vec<wardice::Dice> },
}

#[derive(Serialize, Debug)]
#[serde(tag = "cmd")]
enum Ws_Message_Out {
    On_New_Client { name: String },
    Roll_dice_result { results: Vec<(wardice::Dice, &'static wardice::Face)> },
}

// represent client connection
struct Server {
    client: Client,
    registry: Registry,
}

type Registry = Rc<RefCell<HashMap<Token, Sender>>>;

fn main() {
    println!("Hello, rpgd!");
    let registry = Rc::new(RefCell::new(HashMap::new()));

    impl Handler for Server {
        fn on_open(&mut self, _: Handshake) -> Result<()> {
            println!("connection open {:?}", self.client.sender.token());
            self.registry.borrow_mut().insert(self.client.sender.token(), self.client.sender.clone());
            Ok(())
        }

        fn on_close(&mut self, _: CloseCode, _: &str) {
            println!("connection close {:?}", self.client.sender.token());
            self.registry.borrow_mut().remove(&self.client.sender.token());
        }

        fn on_message(&mut self, msg: Message) -> Result<()> {
            println!("client send message {:?}", msg);

            let req = serde_json::from_slice::<Ws_Message_In>(&msg.clone().into_data());
            match req {
                Ok(r) => println!("deserialize ok => {:?}", r),
                Err(err) => println!("deserialize ko {:?}", err),
            }
            //let req = serde_json::from_str(&msg.into_text().unwrap());
            for (token, client) in self.registry.borrow().iter() {
                println!("send message to {:?}", token);
                let cmsg = msg.clone();
                client.send(cmsg);
            }
            Ok(())
        }
    }

    listen("127.0.0.1:7474", move |connexion| Server {
        client: Client {
            sender: connexion,
            nick: "".to_string(),
            ready: false,
        },
        registry: registry.clone(),
    }).unwrap()
}

struct Client {
    sender: Sender,
    nick: String,
    ready: bool,
}

impl Client {
    fn set_nick(&mut self, nick: String) {
        self.nick = nick
    }

    fn set_is_ready(&mut self, ready: bool) {
        self.ready = ready;
    }
}
