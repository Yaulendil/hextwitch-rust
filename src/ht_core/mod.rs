mod ircv3;
mod events;
mod printing;


use chrono::{DateTime, Utc};
use hexchat::{EatMode, get_network_name};
use parking_lot::Mutex;

use ircv3::Message;


pub struct Sponge {
    pub signature: Option<String>,
    pub value: Option<Message>,
}

impl Sponge {
    /// Place a Message into the Sponge. The previous Message in the Sponge, if
    ///     any, is removed. Takes Ownership of the new Message.
    ///
    /// Input: `Message`
    pub fn put(&mut self, new: Message) {
        self.signature.replace(new.get_signature());
        self.value.replace(new);
    }

    /// Remove the Message from the Sponge, but only if the current Message has
    ///     the Signature specified.
    ///
    /// Input: `&str`
    /// Return: `Option<Message>`
    pub fn pop(&mut self, signature: &str) -> Option<Message> {
        match (&self.signature, &mut self.value) {
            (Some(sig), msg) if sig == signature => msg.take(),
            _ => None,
        }
    }
}


safe_static! {
    static lazy CURRENT: Mutex<Sponge> = Mutex::new(Sponge { signature: None, value: None });
}


//macro_rules! if_twitch {
//    {$b:block} => {
//        match get_network_name() {
//            Some(network) if network.eq_ignore_ascii_case("twitch") => {
//                $b
//            }
//            _ => EatMode::None,
//        }
//    }
//}


pub fn cb_print(_word: &[String], dt: DateTime<Utc>) -> EatMode {
    match get_network_name() {
        Some(network) if network.eq_ignore_ascii_case("twitch") => {
            //  FIXME
            let sig: &str = "asdf";
            //  FIXME

            match CURRENT.lock().pop(sig) {
                None => EatMode::None,
                Some(msg) => {
                    //  TODO: Re-emit Print with User Badges, etc.
//                    EatMode::All
                    EatMode::None
                }
            }
        }
        _ => EatMode::None,
    }
}


/// Handle a Server Message, received by the Hook for "RAW LINE".
pub fn cb_server(_word: &[String], dt: DateTime<Utc>, raw: String) -> EatMode {
    match get_network_name() {
        Some(network) if network.eq_ignore_ascii_case("twitch") => {
            let msg: Message = Message::new(&raw);
            let eat = match msg.command.as_str() {
                //  Chat Messages.
                "PRIVMSG" => {
                    CURRENT.lock().put(msg);
                    EatMode::None
                }
                "WHISPER" => events::whisper(msg),

                "ROOMSTATE" => EatMode::Hexchat,
                "USERSTATE" => events::userstate(msg),

                "USERNOTICE" => events::usernotice(msg),
                "HOSTTARGET" => events::hosttarget(msg),

                //  Moderator Actions.
                "CLEARMSG" => events::clearmsg(msg),
                "CLEARCHAT" => events::clearchat(msg),
                _ => EatMode::None,
            };
            //  TODO: Take `Option<EatMode>` and, if `None`, print `&raw`.
            eat
        }
        _ => EatMode::None,
    }
}
