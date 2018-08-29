#[macro_use]
extern crate serde;
extern crate serde_derive;
extern crate wardice;

use wardice::{Dice, Face};

#[derive(Deserialize, Debug)]
#[serde(tag = "cmd")]
/// List of message receive by the server
/// ###
/// Example
/// SetNick {name: "test"} => {"cmd":"SetNick","nick":"test"}
/// RollDice { dices: [
///     Characteristic,
///     Characteristic,
///     Characteristic,
///     Challenge,
///     Challenge]
///  } => {"cmd":"RollDice", "dices": ["Characteristic","Characteristic","Characteristic","Challenge","Challenge"]}
pub enum WsMessageIn {
    SetNick { nick: String },
    RollDice { dices: Vec<wardice::Dice> },
    NewChatMessage { nick: String, message: String },
}

#[derive(Serialize, Debug)]
#[serde(tag = "cmd")]
/// List of message send by the server
pub enum WsMessageOut {
    OnNewClient {
        nick: String,
    },
    OnLeaveClient {
        nick: String,
    },
    OnRollDice {
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
