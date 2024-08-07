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

// see Ngspice manual table 2.1
sptype_prefix!(
    R, L, C, // resistor, inductor, capacitor
    V, I, // independent voltage/current source
    D, Q, M, X // diode, bjt, mosfet, subcircuit
);

// pub const L: &str = "L";  // what the above macro does for each spice device type
const NET: &str = "";

/// Spice Device Types enumeration
#[derive(Reflect, Debug, Clone)]
pub enum SpDeviceType {
    Gnd,
    V,
    I,
    R,
    L,
    C,
    D,
    Q,
    M,
}

impl SpDeviceType {
    pub fn prefix(&self) -> &'static str {
        match self {
            SpDeviceType::Gnd => V,
            SpDeviceType::V => V,
            SpDeviceType::I => I,
            SpDeviceType::R => R,
            SpDeviceType::L => L,
            SpDeviceType::C => C,
            SpDeviceType::D => D,
            SpDeviceType::Q => Q,
            SpDeviceType::M => M,
        }
    }
}

/// spice types enumeration
#[derive(Reflect, Clone, Debug)]
pub enum SpType {
    Net,
    Device(SpDeviceType),
}

/// schematic element types enumeration
#[derive(Reflect, Clone, Debug)]
pub enum SchType {
    Spice(SpType),
    Port,
    Label,
}

/// spice id to identify a unique device
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct SpDeviceId {
    id: String,
}

impl SpDeviceId {
    pub fn new(id: String) -> Self {
        SpDeviceId { id }
    }
    pub fn get_id(&self) -> &str {
        &self.id
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
