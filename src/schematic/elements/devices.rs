//! Device: defines circuit devices such as resistor, mos, etc.
//!
//! a device is comprised of its graphics, bounding box, ports
//!
//! DeviceType held as asset, create mesh asset if instanced at least once
//! update mesh asset whenever projection scale changes
//! for now, all device types are always loaded
//!
//! device ports are jank until bevy ecs relations
//! show ports on device mesh
//! devicetypes to keep track of list of ports and offsets
//! manually make sure ports visual (mesh) and internals (in device types) match

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use serde::Deserialize;

use crate::schematic::material::SchematicMaterial;

use super::{ElementsRes, SchematicElement};

/// device types, 1 per type, stored as resource
/// needs to contain data about:
/// graphics
/// relative port locations
///

#[derive(Debug, Deserialize)]
struct DevicecTypePort {
    name: String,
    offset: IVec2,
}
impl DevicecTypePort {
    fn new(name: String, offset: IVec2) -> Self {
        Self { name, offset }
    }
}
#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct DeviceType {
    // graphics: DeviceGraphics,
    ports: Box<[DevicecTypePort]>,
}

impl DeviceType {
    pub fn new_resistor() -> Self {
        Self {
            ports: Box::from([
                DevicecTypePort::new("+".into(), IVec2::new(0, 3)),
                DevicecTypePort::new("-".into(), IVec2::new(0, -3)),
            ]),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CirArc {
    center: Vec2,
    radius: f32,
    start_radians: f32, // zero is +x axis
    end_radians: f32,   // pos is ccw
}

/// struct to store data about how to visualize the device
/// a function needs to generate corresponding mesh for every unique instance of this
/// and update the mesh held in assets
#[derive(Debug, Deserialize)]
struct DeviceGraphics {
    /// line is traced from point to point for each inner vector.
    pts: Box<[Box<[Vec2]>]>,
    /// arbitrary number of circles (center, radius, start_radians, end_radians) to be drawn
    cirarcs: Box<[CirArc]>,
    /// device bounds, determines collision
    bounds: Rectangle,
}

/// devices
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Device {
    device_type: Handle<DeviceType>,
}

/// bundle of device components
#[derive(Bundle)]
pub struct DeviceBundle {
    device: Device,
    // tess_data: CompositeMeshData,
    schematic_element: SchematicElement,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
}

impl DeviceBundle {
    pub fn new_resistor(eres: Res<ElementsRes>) -> Self {
        DeviceBundle {
            device: Device {
                device_type: eres.dtype_r.clone(),
            },
            mat: MaterialMesh2dBundle {
                material: eres.mat_dflt.clone(),
                mesh: Mesh2dHandle(eres.mesh_res.clone()),
                ..Default::default()
            },
            schematic_element: eres.se_device.clone(),
        }
    }
}
