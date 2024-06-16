//! stuff for making a netlist from the drawn circuit
//! example netlist:
//!
//! Netlist Created by Circe
//! .model MOSN NMOS level=1
//! .model MOSP PMOS level=1
//! .model DMOD D
//! .model BJTP PNP
//! .model BJTN NPN
//! MN1 net_25 net_25 net_0 net_0 mosn
//! V1 net_22 net_0 3 AC 1 SIN(3.3 1 2k 0 0)
//! R1 net_23 net_22 1k
//! R2 net_25 net_23 1k
//! VGND1 net_0 0 0
//!
//! for the most part, each line describes a device and its port connections
use bevy::prelude::*;

use super::{devices::DevicePorts, SchematicElement, SpDeviceId};

/*
strat: loop through all devices
*/

fn netlist(
    q_devices: Query<(&DevicePorts, &SchematicElement, &SpDeviceId)>,
    qt: Query<&GlobalTransform>,
) {
    for (d, se, spdid) in q_devices.iter() {
        let Some(spdid) = spice_id(se, spdid) else {
            println!("following device did not have a device type");
            dbg!(se, spdid);
            continue;
        };
        for port in d.get_ports().iter() {
            let t = qt.get(*port).unwrap();
            let p = t.translation();
            // find what net p is on, if any
            // or, assign net name to port during pruning
        }
        // followed by device value (e.g. resistance, voltage) and params if any
    }
}

fn spice_id(se: &SchematicElement, spdid: &SpDeviceId) -> Option<String> {
    se.get_dtype()
        .map(|x| x.prefix().to_owned() + spdid.get_id())
}
