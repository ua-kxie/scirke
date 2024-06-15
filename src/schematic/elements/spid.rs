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
    R, L, C, // resistor, inductor, capacitor
    V, I, // independent voltage/current source
    D, Q, M, X // diode, bjt, mosfet
);

// pub const L: &str = "L";  // what the above macro does for each spice device type
pub const NET: &str = "";

/// spice id to identify a unique device or net
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct SpId {
    sptype: &'static str,
    id: String,
}

impl SpId {
    pub fn new(sptype: &'static str, id: String) -> Self {
        SpId { sptype, id }
    }
    pub fn get_id(&self) -> &str {
        &self.id
    }
    pub fn get_spid(&self) -> String {
        self.sptype.to_owned() + &self.id
    }
}
