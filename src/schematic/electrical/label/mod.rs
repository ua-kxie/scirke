//! schematic labels to display text in 2d world
//!

use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::*,
    text::Text2dBounds,
};

use crate::schematic::guides::ZoomInvariant;

use super::{spid::SchType, SchematicElement};

pub const DEFAULT_FONT_SIZE: f32 = 18.0;
// cannot be unbounded without screwing up serialization (contains null)
const DEFAULT_TEXT_BOUNDS: Text2dBounds = Text2dBounds {
    size: Vec2::new(100.0, 1.0),
};

/// component to display param summary
#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct SchematicLabel {
    offset: IVec2,
    parent: Entity,
}
impl SchematicLabel {
    pub fn new(parent: Entity, offset: IVec2) -> Self {
        Self { parent, offset }
    }
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
    se: SchematicElement, // saving of ui nodes broken until 0.14
    zi: ZoomInvariant,
}

impl SchematicLabelBundle {
    pub fn new(parent: Entity, offset: IVec2, value: String) -> Self {
        Self {
            selabel: SchematicLabel { offset, parent },
            text: Text2dBundle {
                text: Text::from_section(
                    value,
                    TextStyle {
                        font_size: DEFAULT_FONT_SIZE,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                text_anchor: bevy::sprite::Anchor::TopLeft,
                text_2d_bounds: DEFAULT_TEXT_BOUNDS,
                ..default()
            },
            zi: ZoomInvariant,
            se: SchematicElement {
                schtype: SchType::Label,
            },
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
            dbg!("2");
            commands.entity(ent).despawn();
            continue;
        };
        let newpos = parent_gt.transform_point(schl.offset.as_vec2().extend(0.0));
        t.translation = newpos;
    }
}
