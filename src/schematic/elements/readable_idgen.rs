//! a module to solve the problem of generate unique readble IDs for use as spice net/device ids
//! e.g. net1, net2, net10, R1, R2, M1, M2
//! device types assign prefix to their ID to distinguish type; e.g. M for mosfet, R for resistor
//! net names can be anything, as such users can rename the entire id
//! users can manually rename an existing ID, but not its prefix for devices. for nets the whole id should be specifiable.
//! would like for it to generate ids which are no longer in use. e.g. net_2 was generated, but the net was deleted
//!
//! sometimes more distinction is desired within a device type; e.g. pmos typically given id prefix MP while nmos given MN
//! the P/N part of the prefix can be changed by the user and does not guarantee collision avoidance
//!
//!

use std::collections::HashMap;

use bevy::{prelude::*, reflect::Reflect, utils::hashbrown::HashSet};

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct IdTracker {
    nets: IdGen,
    v: IdGen,
    i: IdGen,
    r: IdGen,
    l: IdGen,
    c: IdGen,
}

impl IdTracker {
    pub fn new_r_id(&mut self, prefix: &str) -> String {
        self.r.get_id(prefix)
    }
}

/// one of these per recognized spice device prefix (r l c v i m q d etc.)
/// guarantees no collision
#[derive(Reflect, Default)]
struct IdGen {
    library: HashSet<String>,
    history: HashMap<String, u32>,
}

impl IdGen {
    pub fn get_id(&mut self, prefix: &str) -> String {
        // TODO: would rather loop a limited number of times and return a Result<String, E>
        if !self.history.contains_key(prefix) {
            self.history.insert(prefix.into(), 0);
        }
        let watermark = self.history.get_mut(prefix).unwrap();
        loop {
            *watermark += 1;
            let out = format!("{}{}", prefix, watermark);
            if self.library.insert(out.clone()) {
                return out;
            }
        }
    }
    /// attempt to register a new identifier, e.x. a new id set by user
    /// returns true if successful (input is unique), else returns false (input is already taken)
    pub fn register(&mut self, id: &str) -> bool {
        self.library.insert(id.into())
    }
    /// unregister an id no longer in use
    pub fn unregister(&mut self, id: &str) {
        self.library.remove(id);
    }
}
