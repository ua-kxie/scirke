//! Spice Id as a ecs component

use bevy::{prelude::*, reflect::Reflect};

macro_rules! sptype_prefix {
    ($x:ident) => {
        const $x: &str = stringify!($x);  // define const str
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
const NET: &str = "";

/// Spice Device Types enumeration
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub enum SpDeviceType {
    V,
    // I,
    R,
    // L,
    // C,
}

impl SpDeviceType {
    pub fn prefix(&self) -> &'static str {
        match self {
            SpDeviceType::V => V,
            SpDeviceType::R => R,
        }
    }
}

/// spice types enumeration
#[derive(Reflect, Clone)]
pub enum SpType {
    Net,
    Device(SpDeviceType),
}

/// schematic element types enumeration
#[derive(Reflect, Clone)]
pub enum SchType {
    Spice(SpType),
    Port,
}

/// spice id to identify a unique device
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct SpId {
    sptype: SpDeviceType,
    id: String,
}

impl SpId {
    pub fn new(sptype: SpDeviceType, id: String) -> Self {
        SpId { sptype, id }
    }
    pub fn get_sptype(&self) -> &SpDeviceType {
        &self.sptype
    }
    pub fn get_id(&self) -> &str {
        &self.id
    }
    pub fn get_spid(&self) -> String {
        self.sptype.prefix().to_owned() + &self.id
    }
}

/// net ids  identifying a spice net
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct NetId {
    id: String,
}

impl NetId {
    pub fn new(id: String) -> Self {
        Self { id }
    }
    pub fn get_id(&self) -> &str {
        &self.id
    }
}
