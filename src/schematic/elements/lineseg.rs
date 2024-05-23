//! Schematic Element defining a line segment
//! A Line element is comprised of 3 entities: vertex (x2) and lineseg (x1)
//! vertex can be shared between linesegs
//! 
//! isolated vertex are deleted
//! vertex connecting two parallel lineseg are deleted and the linesegs merged
//! lineseg bisected by a vertex gets split in two
//! 
//! a vertex can be moved, which should by extension change the way connected linesegs appear
//! a line can be moved, which will also move the connected vertices
//! 
//! selection and transforms:
//! transforming a selection of lineseg and its vertices will modify lineseg and vertices transforms both, 
//! after which the lineseg would be updated to track the vertices (no change)
//! 
//! back data with petgraph and reflect contents in bevy?
//! - or -
//! put data in bevy, and put into petgraph when its algos are required?


use bevy::{prelude::*, utils::smallvec::SmallVec};

use super::{Pickable, SchematicElement};

/// work with a unit X mesh from (0, 0) -> (1, 0)
#[derive(Component)]
pub struct LineSeg {
    a: Entity,
    b: Entity,
}

struct PickableLineSeg;
impl Pickable for LineSeg {
    fn test(&self) {
        dbg!("woohoo! from lineseg");
    }
}

/// defines the end points of schematic lines
/// global transform should only ever be translated.
#[derive(Component)]
pub struct Vertex {
    branches: SmallVec<[Entity; 8]>  // anything above a three should be circuit schematic warning
}
struct PickableVertex;
impl Pickable for PickableVertex {
    fn test(&self) {
        dbg!("woohoo! from vertex");
    }
}

#[derive(Bundle)]
struct VertexBundle {
    vertex: Vertex,
    schematic_element: SchematicElement,
}

fn transform_lineseg(
    gt: Query<&Transform>,
    mut lines: Query<(Entity, &LineSeg, &mut Transform)>,
    mut commands: Commands,
) {
    for (ent, ls, mut t) in lines.iter_mut() {
        let Ok(a) = gt.get(ls.a) else {
            // a vertex cannot be found
            commands.entity(ent).despawn();
            continue;
        };
        let Ok(b) = gt.get(ls.b) else {
            // a vertex cannot be found
            commands.entity(ent).despawn();
            continue;
        };
        // compute own transform to take unit X from (0, 0) -> (1, 0) to a -> b
        let m10 = b.translation - a.translation;
        *t = Transform::from_translation(a.translation)
        .with_rotation(Quat::from_rotation_z(m10.angle_between(Vec3::Z)))
        .with_scale(Vec3::splat(m10.length()));
    }
}

pub fn setup(
    mut commands: Commands
) {
    let b = VertexBundle{
        vertex: Vertex { branches: SmallVec::new() },
        schematic_element: SchematicElement{ behavior: Box::new(PickableVertex{}) },
    };
    commands.spawn(b);
}

pub fn test(
    q: Query<&SchematicElement, With<Vertex>>,
) {
    for v in q.iter(){
        v.behavior.test();
    }
}

/// system to merge vertices if they overlap - seems expensive
fn merge_vertices(

) {
    
}
