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
/// SetNick {name: "test"} => {"cmd":"SetNick","nick":"test"}
/// RollDice { dices: [
///     Characteristic,
///     Characteristic,
///     Characteristic,
///     Challenge,
///     Challenge]
///  }
/// => {"cmd":"RollDice", "dices": ["Characteristic","Characteristic","Characteristic","Challenge","Challenge"]}
enum WsMessageIn {
    SetNick { nick: String },
    RollDice { dices: Vec<wardice::Dice> },
    NewChatMessage { nick: String, message: String },
}

#[derive(Serialize, Debug)]
#[serde(tag = "cmd")]
enum WsMessageOut {
    OnNewClient {
        nick: String,
    },
    OnLeaveClient {
        nick: String,
    },
    RollDiceResult {
        nick: String,
        results: Vec<(wardice::Dice, &'static wardice::Face)>,
    },
    OnChatMessage {
        nick: String,
        message: String,
    },
    Error {
        message: String,
    },
}

enum Send {
    ToSelf(Message),
    ToOne(Token, Message),
    ToList(Vec<Token>, Message),
    ToOther(Message),
    ToAll(Message),
}

// represent client connection
struct Server {
    client: Client,
    registry: Registry,
}

impl Server {
    fn send(&self, s: Send) {
        match s {
            Send::ToSelf(message) => self.send_to_self(message),
            Send::ToOne(token, message) => self.send_to_one(token, message),
            Send::ToList(tokens, message) => self.send_to_list(tokens, message),
            Send::ToOther(message) => self.send_to_other(message),
            Send::ToAll(message) => self.send_to_all(message),
        }
    }

    fn send_to_self(&self, message: Message) {
        self.client.sender.send(message.clone());
    }

    fn send_to_one(&self, token: Token, message: Message) {
        for (t, client) in self.registry.borrow().iter() {
            if t == &token {
                client.send(message.clone());
                break;
            }
        }
    }

    fn send_to_list(&self, tokens: Vec<Token>, message: Message) {
        for (t, client) in self.registry.borrow().iter() {
            if tokens.contains(t) {
                client.send(message.clone());
            }
        }
    }

    fn send_to_other(&self, message: Message) {
        for (t, client) in self.registry.borrow().iter() {
            if t != &self.client.sender.token() {
                client.send(message.clone());
            }
        }
    }

    fn send_to_all(&self, message: Message) {
        for (t, client) in self.registry.borrow().iter() {
            client.send(message.clone());
        }
    }
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
            if self.client.ready {
                let out = WsMessageOut::OnLeaveClient {
                    nick: self.client.nick.clone(),
                };
                self.send(Send::ToOther(build_message(out)))
            }
        }

        fn on_message(&mut self, msg: Message) -> Result<()> {
            println!("client send message {}", msg);
            match serde_json::from_slice::<WsMessageIn>(&msg.into_data()) {
                Ok(ws_message_in) => {
                    let resp = self.client.parse_ws_message_in(ws_message_in);
                    self.send(resp)
                }

                Err(err) => {
                    let out = WsMessageOut::Error { message: err.to_string() };
                    self.send(Send::ToSelf(build_message(out)))
                }
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

    fn parse_ws_message_in(&mut self, message: WsMessageIn) -> Send {
        match message {
            WsMessageIn::SetNick { nick } => {
                self.set_nick(nick.clone());
                self.set_is_ready(true);
                let out = WsMessageOut::OnNewClient { nick: nick };
                Send::ToAll(build_message(out))
            }
            _ => match self.ready {
                true => parse_message_in(self, message),
                false => {
                    let out = WsMessageOut::Error {
                        message: "Unable to launch commands without login".to_string(),
                    };
                    Send::ToSelf(build_message(out))
                }
            },
        }
    }
}

fn parse_message_in(client: &mut Client, message: WsMessageIn) -> Send {
    match message {
        WsMessageIn::SetNick { nick } => {
            client.set_nick(nick.clone());
            client.set_is_ready(true);
            let out = WsMessageOut::OnNewClient { nick: nick };
            Send::ToAll(build_message(out))
        }
        WsMessageIn::RollDice { dices } => {
            let r = wardice::roll_dices(dices);
            let out = WsMessageOut::RollDiceResult {
                nick: client.nick.clone(),
                results: r,
            };
            Send::ToAll(build_message(out))
        }
        WsMessageIn::NewChatMessage { nick, message } => {
            let out = WsMessageOut::OnChatMessage { nick: nick, message: message };
            Send::ToAll(build_message(out))
        }
    }
}

fn build_message<T>(m: T) -> Message
where
    T: serde::Serialize,
{
    let rjson = serde_json::to_string(&m);
    match rjson {
        Ok(json) => Message::text(json),
        Err(err) => Message::text(format!("{}", err)),
    }
}
