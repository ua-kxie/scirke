//! components for the device_label archetype

use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::*,
};

/// component to display param summary
#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct DeviceLabel {
    offset: IVec2,
    device: Entity,
}
impl MapEntities for DeviceLabel {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.device = entity_mapper.map_entity(self.device);
    }
}
