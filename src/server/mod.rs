pub mod client;
pub mod connection;
pub mod message;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ws::util::Token;
use ws::Sender;

// Store all active client
pub type Registry = Rc<RefCell<HashMap<Token, Sender>>>;
