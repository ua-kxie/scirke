//! schematic elements
//! a schematic element may be reused in a circuit or device designer context (or more)
//! must support: picking by point/ray, by area intersect, by area contained
//! picking by point/ray should only ever mark 1 entity as tentative

use bevy::prelude::*;
use bevy::math::bounding::{Aabb2d, AabbCast2d, RayCast2d};

mod lineseg;
pub use lineseg::LineSegment;
pub use lineseg::LineVertex;
pub use lineseg::create_lineseg;

/// marker component to mark entity as colliding with picking collider
#[derive(Component)]
struct Tentative;

/// marker component to mark entity as selected
#[derive(Component)]
struct Selected;


/// struct describes the picking collider
enum PickingCollider {
    Point(Vec2),
    AreaIntersect(AabbCast2d),
    AreaContains(AabbCast2d),
}

/// event to be sent when the picking collider changes
#[derive(Event)]
struct NewPickingCollider(PickingCollider);

pub struct ElementsPlugin;

impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, lineseg::setup);
        app.add_systems(Update, lineseg::transform_lineseg);
    }
}

// system should work with SchematicElement, GlobalTransform, to determine if colliding with picking collider
// picking system: get all schematicElements, cursor snapped position, and mark tentative elements as such
// based on event: pickingcollider changed
fn pick(
    mut e_new_collider: EventReader<NewPickingCollider>
) {
    let Some(NewPickingCollider(collider)) = e_new_collider.read().last() else {
        return;
    };
    match collider {
        PickingCollider::Point(_) => todo!(),
        PickingCollider::AreaIntersect(_) => todo!(),
        PickingCollider::AreaContains(_) => todo!(),
    }
}

// a line seg should be picked by area intersect if either vertex is contained
// a line seg should be picked by area contains if both vertex is contained
// need different code to run depending on collision target

/// different components that impl a given trait T with functions to compute picking collision
/// add different 

#[derive(Component)]
struct SchematicElement{
    behavior: Box<dyn Pickable + Send + Sync + 'static>,
}

/// Pickable trait to define how elements consider themselves "picked"
trait Pickable {
    fn collides(&self, pc: PickingCollider);
}

// entity wireseg schematicElement(TO)
// entity vertex schematicElement(TO)