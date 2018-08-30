use serde;
use serde_json;
use server::connection::SendResponse;
use server::message::{WsMessageIn, WsMessageOut};
use wardice::roll_dices;
use ws::{Message, Sender};

// Represent a client
pub struct Client {
    // Represent ws connection
    pub sender: Sender,
    // Nickmane of the client
    pub nick: String,
    // if clietn is ready to send receive msg
    pub ready: bool,
}

impl Client {
    // set the nickname of the client
    pub fn set_nick(&mut self, nick: String) {
        info!(target: "client_events", "client {:?} change his nick: {}", self.sender.token() , nick);
        self.nick = nick
    }

    // set the ready of the client
    pub fn set_is_ready(&mut self, ready: bool) {
        info!(target: "client_events", "client {:?} change his status: {}", self.sender.token() , ready);
        self.ready = ready;
    }

    // Parse input ws message
    pub fn parse_ws_message_in(&mut self, message: WsMessageIn) -> SendResponse {
        info!(target: "client_events", "client {:?} receive new message: {:?}", self.sender.token() , message);
        match message {
            WsMessageIn::SetNick { nick } => {
                self.set_nick(nick.clone());
                self.set_is_ready(true);
                let out = WsMessageOut::OnNewClient { nick: nick };
                SendResponse::ToAll(build_message(out))
            }
            _ => match self.ready {
                true => parse_message_in(self, message),
                false => {
                    let out = WsMessageOut::Error {
                        message: "Unable to launch commands without login".to_string(),
                    };
                    SendResponse::ToSelf(build_message(out))
                }
            },
        }
    }
}

fn parse_message_in(client: &mut Client, message: WsMessageIn) -> SendResponse {
    match message {
        WsMessageIn::SetNick { nick } => {
            client.set_nick(nick.clone());
            client.set_is_ready(true);
            let out = WsMessageOut::OnNewClient { nick: nick };
            SendResponse::ToAll(build_message(out))
        }
        WsMessageIn::RollDice { dices } => {
            let r = roll_dices(dices);
            let out = WsMessageOut::OnRollDice {
                nick: client.nick.clone(),
                results: r,
            };
            SendResponse::ToAll(build_message(out))
        }
        WsMessageIn::NewChatMessage { nick, message } => {
            let out = WsMessageOut::OnChatMessage { nick: nick, message: message };
            SendResponse::ToAll(build_message(out))
        }
    }
}

pub fn build_message<T>(m: T) -> Message
where
    T: serde::Serialize,
{
    let rjson = serde_json::to_string(&m);
    match rjson {
        Ok(json) => Message::text(json),
        Err(err) => Message::text(format!("{}", err)),
    }
}
