//! Device: defines circuit devices such as resistor, mos, etc.
//!
//! a device is comprised of its graphics, bounding box, ports
//!
//! DeviceType held as asset, create mesh asset if instanced at least once
//! update mesh asset whenever projection scale changes
//! for now, all device types are always loaded

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use euclid::default::{Box2D, Point2D};
use lyon_tessellation::FillOptions;
use serde::Deserialize;

use crate::{
    bevyon::{self, CompositeMeshData, TessInData},
    schematic::material::SchematicMaterial,
};

use super::{ElementsRes, SchematicElement};

/// device types, 1 per type, stored as resource
/// needs to contain data about:
/// graphics
/// relative port locations
///
#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct DeviceType {
    graphics: DeviceGraphics,
    ports: bool,
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
        // let mut path_builder = bevyon::path_builder();
        // let size = 1.0;
        // path_builder.add_rectangle(
        //     &Box2D {
        //         min: Point2D::splat(-size),
        //         max: Point2D::splat(size),
        //     },
        //     lyon_tessellation::path::Winding::Positive,
        // );
        // let path = Some(path_builder.build());

        // let tessellator_input_data = TessInData {
        //     path,
        //     stroke: None,
        //     fill: Some(FillOptions::DEFAULT),
        // };
        DeviceBundle {
            device: Device::default(),
            // tess_data: CompositeMeshData::from_single_w_color(tessellator_input_data, Color::GRAY),
            mat: MaterialMesh2dBundle {
                material: eres.mat_dflt.clone(),
                mesh: Mesh2dHandle(eres.mesh_res.clone()),
                ..Default::default()
            },
            schematic_element: eres.se_device.clone(),
        }
    }
}
