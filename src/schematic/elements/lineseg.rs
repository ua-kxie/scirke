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
//! render performance: should use gpu instancing of a bunch of unit X lines with based on transform
//! bevy does this automatically for entities which share a mesh and material instance
//! each material type: have a default, picked, selected, both material instance
//! switch material instance if becomes picked or selected or both
//! this way entities are mostly batched automatically
//!
//! back data with petgraph and reflect contents in bevy?
//! - or -
//! put data in bevy, and put into petgraph when its algos are required?
//!
//! simulation should be done in subapp, similar to render, so that app doesn't freeze.
//! but maybe something more basic like a pipeline would do just as well.
//! on simulation command: nets can be sent into petgraph to be simplified? (net names need to be reflected back in schematic)

use bevy::{
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::smallvec::{smallvec, SmallVec},
};

use crate::schematic::{material::{SchematicMaterial, WireMaterial}, tools::PickingCollider};

use super::{ElementsRes, Pickable, SchematicElement};

/// work with a unit X mesh from (0, 0) -> (1, 0)
#[derive(Component)]
pub struct LineSegment {
    a: Entity,
    b: Entity,
}

struct PickableLineSeg;
impl Pickable for LineSegment {
    fn collides(&self, pc: PickingCollider) -> bool {
        match pc {
            PickingCollider::Point(_) => todo!(),
            PickingCollider::AreaIntersect(_) => todo!(),
            PickingCollider::AreaContains(_) => todo!(),
        }
    }
}

/// defines the end points of schematic lines
/// global transform should only ever be translated.
#[derive(Component, Clone)]
pub struct LineVertex {
    branches: SmallVec<[Entity; 8]>, // anything above a three should be circuit schematic warning
}
struct PickableVertex;
impl Pickable for PickableVertex {
    fn collides(&self, pc: PickingCollider) -> bool {
        false  // TODO: make pickable
    }
}
#[derive(Bundle)]
struct VertexBundle {
    vertex: LineVertex,
    schematic_element: SchematicElement,
}

pub fn create_lineseg(
    mut commands: Commands,
    eres: Res<ElementsRes>,
    coords: Vec2,
) -> Entity {
    // vertex and segments have eachothers entity as reference
    // spawn point at cursor position
    let spawn_point =
        SpatialBundle::from_transform(Transform::from_translation(coords.extend(0.0)));
    // segment transform with scale zero since start and end are both at same point
    let spawn_unitx = SpatialBundle::from_transform(Transform::from_scale(Vec3::splat(0.0)));
    let vertex_a = commands.spawn(spawn_point.clone()).id();
    let vertex_b = commands.spawn(spawn_point).id();
    let lineseg = commands.spawn(spawn_unitx).id();

    let mat_bundle = MaterialMesh2dBundle {
        mesh: Mesh2dHandle(
            eres.unitx_mesh.clone().unwrap()
        ),
        material: eres.mat_dflt.clone().unwrap(),
        transform: Transform::from_translation(coords.extend(0.0)).with_scale(Vec3::splat(1.0)),
        ..Default::default()
    };
    let ls = LineSegment {
        a: vertex_a,
        b: vertex_b,
    };
    let v = LineVertex {
        branches: smallvec![lineseg],
    };

    commands.entity(vertex_a).insert(v.clone());
    commands.entity(vertex_a).insert(v);
    commands.entity(lineseg).insert((ls, mat_bundle));

    vertex_b
}

/// this system updates the transforms of all linesegments so that its unitx mesh reflects the position of its defining vertices
/// TODO: for performance, this should only run at specific times
pub fn transform_lineseg(
    gt: Query<&Transform, Without<LineSegment>>,
    mut lines: Query<(Entity, &LineSegment, &mut Transform)>,
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
            .with_rotation(Quat::from_rotation_z(Vec2::X.angle_between(m10.truncate())))
            .with_scale(Vec3::splat(m10.length()));
    }
}

pub fn setup(mut commands: Commands) {
    let b = VertexBundle {
        vertex: LineVertex {
            branches: SmallVec::new(),
        },
        schematic_element: SchematicElement {
            behavior: Box::new(PickableVertex {}),
        },
    };
    commands.spawn(b);
}

pub fn test(q: Query<&SchematicElement, With<LineVertex>>) {
    // for v in q.iter(){
    //     v.behavior.test();
    // }
}

/// system to merge vertices if they overlap - seems expensive
fn merge_vertices() {}
