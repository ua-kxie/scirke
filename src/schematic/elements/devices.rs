//! Device: defines circuit devices such as resistor, mos, etc.
//!
//! a device is comprised of its graphics, bounding box, ports
//!

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use euclid::{Box2D, Point2D};
use lyon_tessellation::FillOptions;

use crate::{
    bevyon::{self, CompositeMeshData, TessInData},
    schematic::material::SchematicMaterial,
};

use super::{ElementsRes, SchematicElement};

// /// device types, 1 per type, stored as resource
// pub struct DeviceType {
//     ports: bool,
// }

/// devices
#[derive(Component, Default)]
pub struct Device {
    // device_type: Handle<DeviceType>
}

#[derive(Bundle)]
pub struct DeviceBundle {
    device: Device,
    tess_data: CompositeMeshData,
    schematic_element: SchematicElement,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
}

impl DeviceBundle {
    pub fn new_resistor(eres: Res<ElementsRes>) -> Self {
        let mut path_builder = bevyon::path_builder();
        let size = 1.0;
        path_builder.add_rectangle(
            &Box2D {
                min: Point2D::splat(-size),
                max: Point2D::splat(size),
            },
            lyon_tessellation::path::Winding::Positive,
        );
        let path = Some(path_builder.build());

        let tessellator_input_data = TessInData {
            path,
            stroke: None,
            fill: Some(FillOptions::DEFAULT),
        };
        DeviceBundle {
            device: Device::default(),
            tess_data: CompositeMeshData::from_single_w_color(tessellator_input_data, Color::GRAY),
            mat: MaterialMesh2dBundle {
                material: eres.mat_dflt.clone(),
                ..Default::default()
            },
            schematic_element: eres.se_device.clone(),
        }
    }
}

