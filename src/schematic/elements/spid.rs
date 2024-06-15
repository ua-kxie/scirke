//! Spice Id as a ecs component

use bevy::{prelude::*, reflect::Reflect};

macro_rules! sptype_prefix {
    ($x:ident) => {
        pub const $x: &str = stringify!($x);  // define const str
    };
    ($x:ident, $($y:ident),+) => {
        sptype_prefix!($x);
        sptype_prefix!($($y),+);
    };
}

sptype_prefix!(
    R, L, C, V, I, // voltage/current source
    D, Q, M // diode, bjt, mosfet
);

// pub const L: &str = "L";  // what the above macro does for each spice device type
pub const NET: &str = "";

/// spice id to identify a unique device or net
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct SpId {
    prefix: &'static str,
    id: String,
}

impl SpId {
    pub fn new(prefix: &'static str, id: String) -> Self {
        SpId { prefix, id }
    }
    pub fn get_id(&self) -> &str {
        &self.id
    }
    pub fn get_spid(&self) -> String {
        self.prefix.to_owned() + &self.id
    }
}