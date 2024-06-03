//! a module to solve the problem of generate unique readble IDs for use as spice net/device ids
//! e.g. net_1, net_2, R1, R2, M1, M2
//! users can manually rename an existing ID and they should be able to name it something like net_12
//! would like for it to generate ids which are no longer in use. e.g. net_2 was generated, but the net was deleted

use bevy::utils::HashSet;

/// id generator
pub struct IdGen {
    prefix: String,
    watermark: u32,
    hashset: HashSet<String>,
}

impl IdGen {
    pub fn get_id(&mut self) -> String {
        // TODO: would rather loop a limited number of times and return a Result<String, E>
        loop {
            self.watermark += 1;
            let out = format!("{}{}", self.prefix, self.watermark);
            if !self.hashset.insert(out.clone()) {
                return out;
            }
        }
    }
    /// attempt to register a new identifier, e.x. a new id set by user
    /// returns true if successful (input is unique), else returns false (input is already taken)
    pub fn register(&mut self, id: String) -> bool {
        self.hashset.insert(id)
    }
}
