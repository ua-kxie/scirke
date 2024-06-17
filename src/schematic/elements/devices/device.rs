//! components for the device archetype

use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

use crate::schematic::material::SchematicMaterial;

use super::{spid, DeviceType, ElementsRes, PickableElement, SchematicElement};

/// component storing device parameters
/// TODO: want to use trait object for this but how to serialize?
#[derive(Component, Reflect)]
#[reflect(Component)]
pub enum DeviceParams {
    Raw(String), // passed directly to ngspice
    Float(f32),
}
impl DeviceParams {
    pub fn spice_param(&self) -> String {
        match &self {
            DeviceParams::Raw(r) => {
                r.clone()
            },
            DeviceParams::Float(f) => {
                f.to_string()
            },
        }
    }
}

// #[derive(Component, Reflect)]
// #[reflect(Component)]
// pub struct DeviceParams0 (
//     Box<dyn Send + Sync + 'static>,  // params trait object
// );

#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct DevicePorts {
    ports: Vec<Entity>,
}
impl DevicePorts {
    pub fn get_ports(&self) -> &Vec<Entity> {
        &self.ports
    }
}
impl MapEntities for DevicePorts {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.ports = self
            .ports
            .iter()
            .map(|e| entity_mapper.map_entity(*e))
            .collect();
    }
}

#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct DeviceLabel {
    label: Entity,
}
impl MapEntities for DeviceLabel {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.label = entity_mapper.map_entity(self.label);
    }
}

#[derive(Bundle)]
pub struct DeviceBundle {
    label: DeviceLabel,
    params: DeviceParams,
    ports: DevicePorts,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
    pe: PickableElement,
    se: SchematicElement,
}

impl DeviceBundle {
    pub fn from_type(
        dtype: &DeviceType,
        eres: &ElementsRes,
        ports: Vec<Entity>,
        label: Entity,
    ) -> Self {
        Self {
            label: DeviceLabel { label },
            params: DeviceParams::Raw("1".to_owned()),
            ports: DevicePorts { ports },
            mat: MaterialMesh2dBundle {
                mesh: dtype.visuals.clone(),
                material: eres.mat_dflt.clone(),
                ..Default::default()
            },
            pe: PickableElement {
                behavior: dtype.collider.clone(),
            },
            se: SchematicElement {
                schtype: spid::SchType::Spice(spid::SpType::Device(dtype.spice_type.clone())),
            },
        }
    }
}
