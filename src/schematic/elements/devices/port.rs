//! components for the port archetype

use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

use crate::schematic::material::SchematicMaterial;

use super::{spid, ElementsRes, SchematicElement};

#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct Port {
    parent_device: Entity,
    offset: IVec2,
}
impl Port {
    pub fn get_parent(&self) -> Entity {
        self.parent_device
    }
    pub fn get_offset(&self) -> IVec2 {
        self.offset
    }
    pub fn get_offset_vec3(&self) -> Vec3 {
        self.offset.extend(0).as_vec3()
    }
}
impl MapEntities for Port {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.parent_device = entity_mapper.map_entity(self.parent_device);
    }
}

#[derive(Bundle)]
pub struct PortBundle {
    port: Port,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
    se: SchematicElement,
}

impl PortBundle {
    pub fn new(deviceid: Entity, offset: IVec2, eres: &ElementsRes) -> Self {
        PortBundle {
            port: Port {
                parent_device: deviceid,
                offset,
            },
            mat: MaterialMesh2dBundle {
                mesh: bevy::sprite::Mesh2dHandle(eres.mesh_port.clone()), // TODO create a mesh for port
                material: eres.mat_dflt.clone(),
                ..Default::default()
            },
            se: SchematicElement {
                schtype: spid::SchType::Port,
            },
        }
    }
}
