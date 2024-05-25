//! schematic elements
//! a schematic element may be reused in a circuit or device designer context (or more)
//! must support: picking by point/ray, by area intersect, by area contained
//! picking by point/ray should only ever mark 1 entity as picked

use bevy::prelude::*;

mod lineseg;
pub use lineseg::create_lineseg;

use super::tools::PickingCollider;

/// marker component to mark entity as colliding with picking collider
#[derive(Component)]
struct picked;

/// marker component to mark entity as selected
#[derive(Component)]
struct Selected;

/// different components that impl a given trait T with functions to compute picking collision
/// add different

#[derive(Component)]
struct SchematicElement {
    behavior: Box<dyn Pickable + Send + Sync + 'static>,
}

/// Pickable trait to define how elements consider themselves "picked"
trait Pickable {
    fn collides(&self, pc: PickingCollider);
}

// entity wireseg schematicElement(TO)
// entity vertex schematicElement(TO)

pub struct ElementsPlugin;

impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, lineseg::setup);
        app.add_systems(Update, lineseg::transform_lineseg);
    }
}

// system to apply selected/picked marker components
fn picking(
    
) {

}

// a line seg should be picked by area intersect if either vertex is contained
// a line seg should be picked by area contains if both vertex is contained
// need different code to run depending on collision target
