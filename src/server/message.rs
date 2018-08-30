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
    // Ws message to set nickname
    SetNick { nick: String },
    // Ws message to roll some dice
    RollDice { dices: Vec<Dice> },
    // Ws message to send new message
    NewChatMessage { nick: String, message: String },
}

#[derive(Serialize, Debug)]
#[serde(tag = "cmd")]
/// List of message send by the server
pub enum WsMessageOut {
    // Ws message when client join
    OnNewClient { nick: String },
    // Ws message when client leave
    OnLeaveClient { nick: String },
    // Ws message when a client toll dices
    OnRollDice { nick: String, results: Vec<(Dice, &'static Face)> },
    // Ws message when a new message was send
    OnChatMessage { nick: String, message: String },
    // Ws message when an error occured
    Error { message: String },
}
