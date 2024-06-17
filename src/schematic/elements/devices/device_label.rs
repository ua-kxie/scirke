//! components for the device_label archetype

use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::*,
};

/// component to display param summary
#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct SchematicLabel {
    offset: IVec2,
    parent: Entity,
}
impl MapEntities for SchematicLabel {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.parent = entity_mapper.map_entity(self.parent);
    }
}

#[derive(Bundle)]
pub struct SchematicLabelBundle {
    selabel: SchematicLabel,
    text: Text2dBundle,
    // se: SchematicElement,  // saving of ui nodes broken until 0.14
}

impl SchematicLabelBundle {
    pub fn new(parent: Entity, offset: IVec2, value: String) -> Self {
        Self {
            selabel: SchematicLabel { offset, parent },
            text: Text2dBundle {
                text: Text::from_section(
                    value,
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                transform: Transform::from_scale(Vec3::splat(0.1)),
                ..default()
            },
            // se: SchematicElement {schtype: SchType::Label}
        }
    }
}

/// system to update location of SchmaticLabels
pub fn sch_label_update(
    mut q_schlabels: Query<(Entity, &SchematicLabel, &mut Transform)>,
    qt: Query<&GlobalTransform>,
    mut commands: Commands,
) {
    for (ent, schl, mut t) in q_schlabels.iter_mut() {
        let Ok(parent_gt) = qt.get(schl.parent) else {
            commands.entity(ent).despawn();
            continue;
        };
        let newpos = parent_gt.transform_point(schl.offset.as_vec2().extend(0.0));
        t.translation = newpos;
    }
}
