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
use std::{collections::HashMap, fs};

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_egui::egui::Color32;

use crate::schematic::ui::console::PrintConsoleLine;

use super::{
    devices::{DeviceParams, DevicePorts},
    label::SchematicLabelBundle,
    nets::{Port, PortLabel},
    spmanager::SPRes,
    NetId, SchematicElement, SpDeviceId,
};

#[derive(Resource, Deref, DerefMut)]
pub struct SimAcHz(pub f32);

impl Default for SimAcHz {
    fn default() -> Self {
        Self(60.0)
    }
}

#[derive(Event)]
pub struct Netlist;

pub struct NetlistPlugin;

impl Plugin for NetlistPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Netlist>();
        app.add_systems(
            PreUpdate,
            (netlist, pksim)
                .chain()
                .run_if(input_just_pressed(KeyCode::Space)),
        ); // preupdate: run on schematic that has been seen
        app.init_resource::<SimAcHz>();
    }
}

fn netlist(
    q_devices: Query<(&DevicePorts, &DeviceParams, &SchematicElement, &SpDeviceId)>,
    q_nid: Query<&NetId>,
) {
    let mut netlist = String::from("Netlist Created by Sircke\n");
    for (d, params, se, spdid) in q_devices.iter() {
        let Some(spdid) = spice_id(se, spdid) else {
            let s = format!(
                "netlist failed: this device did not have a device type {:?}, {:?}",
                se, spdid
            );
            error!("netlist failed: this device did not have a device type");
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

fn pksim(
    mut e_console_rgstr: EventWriter<PrintConsoleLine>,
    q_ports: Query<(Entity, &NetId), With<Port>>,
    q_labeled_ports: Query<(Entity, &PortLabel), With<Port>>,
    spres: Res<SPRes>,
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    sim_achz: Res<SimAcHz>,
) {
    // clear all port labels
    for (e, p) in q_labeled_ports.iter() {
        commands.entity(p.get_label_entity()).despawn();
        commands.entity(e).remove::<PortLabel>();
    }

    // run sim
    e_console_rgstr.send(PrintConsoleLine::new(
        "source out/netlist.cir".to_owned(),
        Color32::GRAY,
    ));
    spres.command("source out/netlist.cir");
    let cmd;
    if keys.pressed(KeyCode::ControlLeft) {
        // run acop
        let hz = **sim_achz;
        // ac step_type step_size start end (start and end are inclusive)
        // since dcop, start and end can be same
        cmd = format!("ac lin 0 {} {}", hz, hz);
    } else {
        // run dcop
        cmd = "op".to_owned();
    }
    // send to ngspice
    spres.command(&cmd);
    // send to console for posterity
    e_console_rgstr.send(PrintConsoleLine::new(cmd, Color32::GRAY));

    // get results, display as port labels
    if let Some(pkvecvaluesall) = spres.get_spm().vecvals_pop() {
        let mut results = HashMap::<String, String>::new();
        for v in pkvecvaluesall.vecsa {
            if v.is_complex {
                let cv = num::complex::Complex::new(v.creal, v.cimag);
                results.insert(v.name, format! {"{:+.2e}", cv});
            } else {
                results.insert(v.name, format! {"{:+.2e}", v.creal});
            }
        }
        for (ent, netid) in q_ports.iter() {
            let val = results.get(netid.get_id()).unwrap();
            insert_new_label(ent, &mut commands, val.clone());
        }
    }
}

fn insert_new_label(parent: Entity, commands: &mut Commands, val: String) {
    let label_entity = commands
        .spawn(SchematicLabelBundle::new(parent, IVec2::splat(0), val))
        .id();
    commands.entity(parent).insert(PortLabel::new(label_entity));
}
