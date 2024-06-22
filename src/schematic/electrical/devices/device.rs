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
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub enum DeviceParams {
    Raw(String), // passed directly to ngspice
    Float(f32),
}
impl DeviceParams {
    pub fn spice_param(&self) -> String {
        match &self {
            DeviceParams::Raw(r) => r.clone(),
            DeviceParams::Float(f) => f.to_string(),
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
        dtype: DeviceType,
        eres: &ElementsRes,
        ports: Vec<Entity>,
        label: Entity,
    ) -> Self {
        Self {
            label: DeviceLabel { label },
            params: dtype.params,
            ports: DevicePorts { ports },
            mat: MaterialMesh2dBundle {
                mesh: dtype.visuals,
                material: eres.mat_dflt.clone(),
                ..Default::default()
            },
            pe: PickableElement {
                behavior: dtype.collider,
            },
            se: SchematicElement {
                schtype: spid::SchType::Spice(spid::SpType::Device(dtype.spice_type)),
            },
        }
    }
}

pub fn update_device_param_labels(q: Query<(&DeviceParams, &DeviceLabel)>, mut commands: Commands) {
    for (p, l) in q.iter() {
        commands
            .get_entity(l.label)
            .unwrap()
            .insert(Text::from_section(
                p.spice_param(),
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
    }
}
