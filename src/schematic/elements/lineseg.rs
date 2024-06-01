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

use crate::schematic::{guides::ZoomInvariant, tools::PickingCollider};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::smallvec::{smallvec, SmallVec},
};
use euclid::default::{Box2D, Point2D};

use super::{ElementsRes, Pickable, SchematicElement, Selected};

/// work with a unit X mesh from (0, 0) -> (1, 0)
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct LineSegment {
    a: Entity,
    b: Entity,
}

struct PickableLineSeg {
    bounds: Box2D<f32>,
}
impl Default for PickableLineSeg {
    fn default() -> Self {
        Self {
            bounds: Box2D::from_points([Point2D::splat(0.0), Point2D::new(1.0, 0.0)])
                .inflate(0.1, 0.1),
        }
    }
}
impl Pickable for PickableLineSeg {
    fn collides(&self, pc: &PickingCollider, gt: Mat4) -> bool {
        match pc {
            PickingCollider::Point(p) => {
                let p1 = gt.transform_point3(p.extend(0.0));
                let (s, _, _) = gt.to_scale_rotation_translation();
                Box2D::from_points([Point2D::splat(0.0), Point2D::new(1.0, 0.0)])
                    // inflate proportional to inverse transform scale so that longer lines dont get bigger hit boxes
                    .inflate(-s.y * 0.5, s.y * 0.5)
                    .contains_inclusive(Point2D::new(p1.x, p1.y))
            }
            PickingCollider::AreaIntersect(_) => false,
            PickingCollider::AreaContains(_) => false,
        }
    }
}

/// defines the end points of schematic lines
/// global transform should only ever be translated.
#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct LineVertex {
    branches: SmallVec<[Entity; 8]>, // anything above a three should be circuit schematic warning
}

#[derive(Clone, Default)]
struct PickableVertex;
impl Pickable for PickableVertex {
    fn collides(&self, pc: &PickingCollider, gt: Mat4) -> bool {
        match pc {
            PickingCollider::Point(p) => {
                let p1 = gt.transform_point3(p.extend(0.0));
                Box2D::from_points([Point2D::splat(-0.5), Point2D::splat(0.5)])
                    .contains_inclusive(Point2D::new(p1.x, p1.y))
            }
            PickingCollider::AreaIntersect(_) => false,
            PickingCollider::AreaContains(_) => false,
        }
    }
}

pub fn lsse() -> SchematicElement {
    SchematicElement {
        behavior: Box::new(PickableLineSeg::default()),
    }
}

pub fn lvse() -> SchematicElement {
    SchematicElement {
        behavior: Box::new(PickableVertex::default()),
    }
}
#[derive(Bundle)]
struct VertexBundle {
    vertex: LineVertex,
    schematic_element: SchematicElement,
}

pub fn create_lineseg(mut commands: Commands, eres: Res<ElementsRes>, coords: Vec2) -> Entity {
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
        mesh: Mesh2dHandle(eres.mesh_unitx.clone().unwrap()),
        material: eres.mat_dflt.clone().unwrap(),
        transform: Transform::from_translation(coords.extend(0.0)).with_scale(Vec3::splat(1.0)),
        ..Default::default()
    };
    let ls = (
        LineSegment {
            a: vertex_a,
            b: vertex_b,
        },
        mat_bundle,
        SchematicElement {
            behavior: Box::new(PickableLineSeg::default()),
        },
    );
    let v = (
        LineVertex {
            branches: smallvec![lineseg],
        },
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(eres.mesh_dot.clone().unwrap()),
            material: eres.mat_dflt.clone().unwrap(),
            transform: Transform::from_translation(coords.extend(0.0)),
            ..Default::default()
        },
        ZoomInvariant,
    );

    commands.entity(vertex_a).insert((
        v.clone(),
        SchematicElement {
            behavior: Box::new(PickableVertex::default()),
        },
    ));
    commands.entity(vertex_b).insert((
        v,
        SchematicElement {
            behavior: Box::new(PickableVertex::default()),
        },
    ));
    commands.entity(lineseg).insert(ls);

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

/// system to prune line segs and vertices
/// deletes vertices with no branches or segments missing a vertex
pub fn prune(
    mut commands: Commands,
    qls: Query<(Entity, &LineSegment)>,
    mut qlv: Query<(Entity, &mut LineVertex)>,
) {
    for (e, ls) in qls.iter() {
        if commands.get_entity(ls.a).is_none() || commands.get_entity(ls.b).is_none() {
            commands.entity(e).despawn();
        }
    }
    for (e, mut lv) in qlv.iter_mut() {
        lv.branches = lv
            .branches
            .iter()
            .filter_map(|ls| {
                if commands.get_entity(*ls).is_some() {
                    Some(*ls)
                } else {
                    None
                }
            })
            .collect();
        if lv.branches.is_empty() {
            commands.entity(e).despawn();
        }
    }
}

/// extend selection too line segs to connected vertices
pub fn extend_selection(q: Query<&LineSegment, Changed<Selected>>, mut commands: Commands) {
    for ls in q.iter() {
        commands.entity(ls.a).insert(Selected);
        commands.entity(ls.b).insert(Selected);
    }
}
