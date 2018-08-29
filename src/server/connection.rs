extern crate log;
extern crate ws;

use serde_json;
use ws::util::Token;
use ws::{listen, CloseCode, Handler, Handshake, Message, Result, Sender};

use server::client::{build_message, Client};
use server::message::{WsMessageIn, WsMessageOut};
use server::Registry;

// Represent client connection
pub struct Connection {
    client: Client,
    registry: Registry,
}

// Represent all kind of response
pub enum SendResponse {
    ToSelf(Message),
    ToOne(Token, Message),
    ToList(Vec<Token>, Message),
    ToOther(Message),
    ToAll(Message),
}

impl Connection {
    // Parse response message
    fn send(&self, s: SendResponse) {
        match s {
            SendResponse::ToSelf(message) => self.send_to_self(message),
            SendResponse::ToOne(token, message) => self.send_to_ont(token, message),
            SendResponse::ToList(tokens, message) => self.send_to_list(tokens, message),
            SendResponse::ToOther(message) => self.send_to_other(message),
            SendResponse::ToAll(message) => self.send_to_all(message),
        }
    }

    // Send message to self
    fn send_to_self(&self, message: Message) {
        let ct = self.client.sender.token();
        info!(target: "connection_events", "client {} send message to self: message => {}", ct , message);
        send_message(ct, self.client, message);
    }

    // Send message to one client
    fn send_to_one(&self, token: Token, message: Message) {
        let ct = self.client.sender.token();
        info!(target: "connection_events", "client {} send message to one: token => {} message => {}", ct, token , message);
        for (t, client) in self.registry.borrow().iter() {
            if t == &token {
                info!(target: "connection_events", "client {} send message to one: dest found {}", ct, t);
                send_message(ct, client, message);
                break;
            }
        }
    }

    // Send message to a list of clients
    fn send_to_list(&self, tokens: Vec<Token>, message: Message) {
        let ct = self.client.sender.token();
        info!(target: "connection_events", "client {} send message to list: tokens => {} message => {}", ct, tokens , message);
        for (t, client) in self.registry.borrow().iter() {
            if tokens.contains(t) {
                info!(target: "connection_events", "client {} send message to list: dest found {}", ct, t);
                send_message(ct, client, message);
            }
        }
    }

    // Send message to other clients
    fn send_to_other(&self, message: Message) {
        let ct = self.client.sender.token();
        info!(target: "connection_events", "client {} send message to other: message => {}", ct , message);
        for (t, client) in self.registry.borrow().iter() {
            if t != &self.client.sender.token() {
                info!(target: "connection_events", "client {} send message to other: dest found {}", ct, t);
                send_message(ct, client, message);
            }
        }
    }

    // Send message to all clients
    fn send_to_all(&self, message: Message) {
        let ct = self.client.sender.token();
        info!(target: "connection_events", "client {} send message to all: message => {}", ct , message);
        for (t, client) in self.registry.borrow().iter() {
            info!(target: "connection_events", "client {} send message to all: dest found {}", ct, t);
            send_message(ct, client, message);
        }
    }
}

// Send message to client
fn send_message(sender: Token, client: Client, message: Message) {
    match client.sender.send(message.clone()) {
        Ok() => (),
        Err(err) => {
            error!(target: "connection_events", "client {} send message to {}: error => {}", sender, client.sender.token(), err);
        }
    }
}

impl Handler for Connection {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        info!(target: "client_events", "client {} open a connection", self.client.sender.token());
        self.registry.borrow_mut().insert(self.client.sender.token(), self.client.sender.clone());
        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        info!(target: "client_events", "client {} close is connection", self.client.sender.token());
        self.registry.borrow_mut().remove(&self.client.sender.token());
        if self.client.ready {
            let out = WsMessageOut::OnLeaveClient {
                nick: self.client.nick.clone(),
            };
            self.send(Send::ToOther(build_message(out)))
        }
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        info!(target: "client_events", "client {} send a message", self.client.sender.token(), msg);
        match serde_json::from_slice::<WsMessageIn>(&msg.into_data()) {
            Ok(ws_message_in) => {
                info!(target: "client_events", "client {} unmarshall message succeed {}", self.client.sender.token(), ws_message_in);
                let resp = self.client.parse_ws_message_in(ws_message_in);
                self.send(resp)
            }

            Err(err) => {
                error!(target: "client_events", "client {} unmarshall message fail {}", self.client.sender.token(), err);
                let out = WsMessageOut::Error { message: err.to_string() };
                self.send(Send::ToSelf(build_message(out)))
            }
        }
        Ok(())
    }
}
