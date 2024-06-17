//! components for the port archetype

use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

use crate::schematic::material::SchematicMaterial;

use super::{spid, DevicePorts, ElementsRes, SchematicElement};

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

pub fn update_port_location(
    q: Query<(&GlobalTransform, &DevicePorts)>,
    mut q_p: Query<(Entity, &mut Transform, &Port)>,
    mut commands: Commands,
) {
    // delete all ports without valid parent device
    for (e, _, port) in q_p.iter() {
        if commands.get_entity(port.get_parent()).is_none() {
            commands.entity(e).despawn();
        }
    }
    // update position of ports
    for (device_gt, d) in q.iter() {
        for port_entity in d.get_ports().iter() {
            let Ok((_, mut port_t, port)) = q_p.get_mut(*port_entity) else {
                continue;
            };
            let mut newt = device_gt.transform_point(port.get_offset_vec3());
            newt.z = 0.01;
            port_t.translation = newt;
        }
    }
}