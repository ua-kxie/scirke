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
use std::fs;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use super::{
    devices::{DeviceParams, DevicePorts},
    NetId, SchematicElement, SpDeviceId,
};

#[derive(Event)]
pub struct Netlist;

pub struct NetlistPlugin;

impl Plugin for NetlistPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Netlist>();
        app.add_systems(
            PreUpdate,
            netlist.run_if(input_just_pressed(KeyCode::Space)),
        ); // preupdate: run on schematic that has been seen
    }
}

fn netlist(
    q_devices: Query<(&DevicePorts, &DeviceParams, &SchematicElement, &SpDeviceId)>,
    q_nid: Query<&NetId>,
) {
    let mut netlist = String::from("Netlist Created by Sircke\n");
    for (d, params, se, spdid) in q_devices.iter() {
        let Some(spdid) = spice_id(se, spdid) else {
            println!("following device did not have a device type");
            dbg!(se, spdid);
            continue;
        };
        // push device id
        netlist.push_str(&spdid);
        netlist.push_str(" ");
        // push net id for each port
        for port in d.get_ports().iter() {
            let net = q_nid.get(*port).unwrap().get_id();
            netlist.push_str(net);
            netlist.push_str(" ");
        }
        // followed by device value (e.g. resistance, voltage) and params if any
        netlist.push_str(&params.spice_param());
        netlist.push_str("\n");
    }
    if netlist == String::from("Netlist Created by Sircke\n") {
        // empty netlist
        netlist.push_str("V_0 0 n1 0"); // give it something so spice doesnt hang
    }
    fs::write("out/netlist.cir", netlist.as_bytes()).expect("Unable to write file");
}

fn spice_id(se: &SchematicElement, spdid: &SpDeviceId) -> Option<String> {
    se.get_dtype()
        .map(|x| x.prefix().to_owned() + spdid.get_id())
}
