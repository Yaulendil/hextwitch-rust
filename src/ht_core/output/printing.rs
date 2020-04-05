use std::collections::HashMap;

use hexchat::{get_current_channel, print_event_to_channel, PrintEvent};
use parking_lot::RwLock;

use super::{
    super::irc::split_at_first,
    tabs::TABCOLORS,
};


/// Channel Events: Subscriptions, Highlighted Messages, etc.
pub const EVENT_ALERT: PrintEvent = PrintEvent::WHOIS_SERVER_LINE;
/// Links to other Channels, like Hosting.
pub const EVENT_CHANNEL: PrintEvent = PrintEvent::CHANNEL_URL;
/// Red "error" text: Things going wrong, or people being banned.
pub const EVENT_ERR: PrintEvent = PrintEvent::SERVER_ERROR;
/// Typical events.
pub const EVENT_NORMAL: PrintEvent = PrintEvent::MOTD;
/// Reward Events: Bits and custom Points Rewards.
pub const EVENT_REWARD: PrintEvent = PrintEvent::WHOIS_AUTHENTICATED;


/// Echo: Print an event to HexChat in the current Channel, and color the tab.
///
/// Input: `PrintEvent`, `&[impl AsRef<str>]`, `u8`
pub fn echo(event: PrintEvent, args: &[impl AsRef<str>], tab_color: u8) {
    let channel = get_current_channel();

    print_event_to_channel(&channel, event, args);
    TABCOLORS.write().color(channel, tab_color);
}


/// BADGE_NONE: A placeholder Badge string for the User when a UserState has not
///     been received.
const BADGE_NONE: &str = "_ ";
/// BITS: Badge characters for Bits. If a User has a Bits Badge, the User is
///     given the `char` corresponding to the last value found here which is
///     LESS THAN OR EQUAL TO the Rank of the Badge.
/// NOTE: if any value here is not greater than the previous one, it and
///     subsequent pairs will not be considered in the correct order.
static BITS: &[(usize, char)] = &[
    (0, '▴'),
    (100, '⬧'),
    (1_000, '⬠'),
    (5_000, '⬡'),
    (10_000, '🟋'),
    // (25_000, '?'),
    // (50_000, '?'),
    // (75_000, '?'),
    (100_000, '🟎'),
    // (200_000, '?'),
    // (300_000, '?'),
    // (400_000, '?'),
    // (500_000, '?'),
    // (600_000, '?'),
    // (700_000, '?'),
    // (800_000, '?'),
    // (900_000, '?'),
    // (1_000_000, '?'),
];
/// SUBS: Badge characters for Subscribers. If a User has a Sub Badge, the User
///     is given the `char` corresponding to the last value found here which is
///     LESS THAN OR EQUAL TO the Rank of the Badge.
/// NOTE: if any value here is not greater than the previous one, it and
///     subsequent pairs will not be considered in the correct order.
static SUBS: &[(usize, char)] = &[
    (0, '①'),
    (3, '③'),
    (6, '⑥'),
    (9, '⑨'),
    (12, 'ⅰ'),
    (24, 'ⅱ'),
    (36, 'ⅲ'),
    (48, 'ⅳ'),
    (60, 'ⅴ'),
    (72, 'ⅵ'),
    (84, 'ⅶ'),
    (96, 'ⅷ'),
    (108, 'ⅸ'),
    (120, 'ⅹ'),
    (132, 'ⅺ'),
    (144, 'ⅻ'),
];
// /// GIFTS: Badge characters for Subscription Gifters. If a User has a Gifter
// /// Badge, the User is given the `char` corresponding to the last value found
// /// here which is LESS THAN OR EQUAL TO the Rank of the Badge.
// /// NOT E: if any value here is not greater than the previous one, it and
// ///     subsequent pairs will not be considered in the correct order.
// static GIFTS: &[(usize, char)] = &[
//     (0, ':'),
//     // (5, '?'),
//     // (10, '?'),
//     // (25, '?'),
//     // (50, '?'),
//     // (100, '?'),
//     // (250, '?'),
//     // (500, '?'),
//     // (1_000, '?'),
// ];


fn get_badge(class: &str, rank: &str) -> Option<char> {
    match class {
        "broadcaster" /**/ => Some('🜲'),
        "staff"       /**/ => Some('🜨'),
        "admin"       /**/ => Some('🜶'),
        "moderator"   /**/ => Some('🗡'),  // ⛨?
        "subscriber"  /**/ => highest(rank.parse().unwrap_or(0), &SUBS),
        "vip"         /**/ => Some('⚑'),
        "founder"     /**/ => Some('ⲷ'),
        "sub-gift-leader"  => Some('⁘'),
        // "sub-gifter"  /**/ => highest(rank.parse().unwrap_or(0), &GIFTS),
        "sub-gifter"  /**/ => Some(':'),
        "bits-leader" /**/ => Some('❖'),
        "bits"        /**/ => highest(rank.parse().unwrap_or(0), &BITS),
        "partner"     /**/ => Some('✓'),
        "turbo"       /**/ => Some('+'),
        "premium"     /**/ => Some('±'),
        _ => None,
    }
}


fn highest(max: usize, seq: &[(usize, char)]) -> Option<char> {
    let mut out: Option<char> = None;

    for &(rank, icon) in seq {
        if rank <= max {
            out.replace(icon);
        } else {
            break;
        }
    }

    out
}


/// Badges: A Struct storing the Input and Output of the process of breaking
///     down a badge value. This effectively serves the purpose of a Cached
///     Function.
#[derive(Default)]
pub struct Badges {
    input: String,
    pub output: String,
}

impl std::str::FromStr for Badges {
    type Err = ();

    /// Break down a string to find the final set of characters. The original
    ///     will be stored.
    ///
    /// Input: `&str`
    /// Return: `Result<Badges, ()>`
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut output: String = String::new();

        for pair in input.split(",") {
            let (class, rank) = split_at_first(pair, "/");

            if let Some(c) = get_badge(class, rank) {
                output.push(c);
            }
        }

        if !output.is_empty() { output.push(' '); }
        Ok(Self { input: String::from(input), output })
    }
}


/// States: Effectively a Box for a HashMap. Stores the Badges for the User in
///     each Channel.
pub struct States(HashMap<String, Badges>);

impl States {
    fn new() -> Self { Self(HashMap::new()) }

    /// Get the Badges for the User in a given Channel.
    ///
    /// Input: `&str`
    /// Return: `&str`
    pub fn get(&self, channel: &str) -> &str {
        match self.0.get(channel) {
            Some(badges) => &badges.output,
            None => BADGE_NONE,
        }
    }

    /// Set the Badges for the User in a given Channel. This is mostly just a
    ///     guarded passthrough to the `HashMap::insert()` of the internal map,
    ///     but with one significant difference: If the current value for the
    ///     given Channel in the Map was created from the same input as has been
    ///     given here, the input is NOT evaluated again.
    ///
    /// Input: `String`, `String`
    pub fn set(&mut self, channel: String, new: String) {
        match self.0.get_mut(&channel) {
            Some(old) if new == old.input => {}  // Channel is in Map, with the same Badges.
            guard => {
                let badges: Badges = new.parse().unwrap_or_default();

                if let Some(b) = guard {
                    *b = badges;
                } else {
                    self.0.insert(channel, badges);
                }
            }
        }
    }
}

safe_static! {
    pub static lazy USERSTATE: RwLock<States> = RwLock::new(States::new());
}