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

use std::collections::{HashMap, HashSet};

use super::{ElementsRes, Pickable, Preview, SchematicElement, Selected};
use crate::schematic::{
    guides::ZoomInvariant, material::SchematicMaterial, tools::PickingCollider,
};
use bevy::{
    ecs::{
        entity::{Entity, EntityMapper, MapEntities},
        reflect::{ReflectComponent, ReflectMapEntities},
    },
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::smallvec::{smallvec, SmallVec},
};
use euclid::default::{Box2D, Point2D};

/// work with a unit X mesh from (0, 0) -> (1, 0)
#[derive(Component, Reflect, PartialEq, Eq, Hash, Clone)]
#[reflect(Component, MapEntities)]
pub struct LineSegment {
    a: Entity,
    b: Entity,
}
impl MapEntities for LineSegment {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.a = entity_mapper.map_entity(self.a);
        self.b = entity_mapper.map_entity(self.b);
    }
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
                    .inflate(-s.y * 0.5, s.y * 0.1)
                    .contains_inclusive(Point2D::new(p1.x, p1.y))
            }
            PickingCollider::AreaIntersect(_) => false,
            PickingCollider::AreaContains(_) => false,
        }
    }
}

/// defines the end points of schematic lines
/// global transform should only ever be translated.
#[derive(Component, Clone, Reflect, Default)]
#[reflect(Component, MapEntities)]
pub struct LineVertex {
    branches: SmallVec<[Entity; 8]>, // anything above a three should be circuit schematic warning
}

impl MapEntities for LineVertex {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        for entity in &mut self.branches {
            *entity = entity_mapper.map_entity(*entity);
        }
    }
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

#[derive(Bundle)]
struct LineSegBundle {
    ls: LineSegment,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
    se: SchematicElement,
}

impl LineSegBundle {
    fn new(eres: &ElementsRes, a: (Entity, Vec3), b: (Entity, Vec3)) -> Self {
        let m10 = b.1 - a.1;
        let transform = Transform::from_translation(a.1)
            .with_rotation(Quat::from_rotation_z(Vec2::X.angle_between(m10.truncate())))
            .with_scale(Vec3::splat(m10.length()));

        let mat = MaterialMesh2dBundle {
            mesh: Mesh2dHandle(eres.mesh_unitx.clone().unwrap()),
            material: eres.mat_dflt.clone().unwrap(),
            transform: transform,
            ..Default::default()
        };
        let ls = LineSegment { a: a.0, b: b.0 };
        let se = SchematicElement {
            behavior: Box::new(PickableLineSeg::default()),
        };
        Self { ls, mat, se }
    }
}

/// creates a preview (missing schematicElement marker) lineseg from src to dst
/// a lineseg consists of 3 entities: 2 vertices and 1 segment.
pub fn create_preview_lineseg(
    commands: &mut Commands,
    eres: &Res<ElementsRes>,
    src: Vec3,
    dst: Vec3,
) {
    // vertex and segments have eachothers entity as reference
    // segment transform with scale zero since start and end are both at same point
    let vertex_a = commands
        .spawn(SpatialBundle::from_transform(Transform::from_translation(
            src,
        )))
        .id();
    let vertex_b = commands
        .spawn(SpatialBundle::from_transform(Transform::from_translation(
            dst,
        )))
        .id();
    let ls = LineSegBundle::new(eres, (vertex_a, src), (vertex_b, dst));
    let lineseg = commands.spawn(ls).id();

    let vsrc = (
        LineVertex {
            branches: smallvec![lineseg],
        },
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(eres.mesh_dot.clone().unwrap()),
            material: eres.mat_dflt.clone().unwrap(),
            transform: Transform::from_translation(src),
            ..Default::default()
        },
        ZoomInvariant,
    );
    let vdst = (
        LineVertex {
            branches: smallvec![lineseg],
        },
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(eres.mesh_dot.clone().unwrap()),
            material: eres.mat_dflt.clone().unwrap(),
            transform: Transform::from_translation(dst),
            ..Default::default()
        },
        ZoomInvariant,
    );

    commands.entity(vertex_a).insert((
        vsrc,
        SchematicElement {
            behavior: Box::new(PickableVertex::default()),
        },
        Preview,
    ));
    commands.entity(vertex_b).insert((
        vdst,
        SchematicElement {
            behavior: Box::new(PickableVertex::default()),
        },
        Preview,
    ));
    commands.entity(lineseg).insert((Preview,));
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

/// extend selection too line segs to connected vertices
pub fn extend_selection(q: Query<&LineSegment, Changed<Selected>>, mut commands: Commands) {
    for ls in q.iter() {
        commands.entity(ls.a).insert(Selected);
        commands.entity(ls.b).insert(Selected);
    }
}

/// full functionality:
/// this function is called whenever schematic is changed. Ensures all connected nets have the same net name, overlapping segments are merged, etc.
/// extra_vertices are coordinates where net segments should be bisected (device ports)
///
/// step 0: ports
/// add a net vertex at location of every device port (or maybe every port should just register as such)
///
/// step 0.1: merge vertices on top of eachother
///
/// step 1: bisect
/// for all vertices: bisect any line seg going over it
/// then merge any overlapping segments
///
/// step 2: cull
/// delete any segments missing one or both end point(s)
/// delete any vertices by itself, not overlapping a device port
///
/// step 3: net labeling
/// get subgraph/nets, assign unique id string to each
pub fn prune(world: &mut World) {
    // authors note: I absolutely hate this implementation
    merge_overlapped_vertex(world);
    bisect_merge(world);
    cull(world);
}

/// this function merges vertices occupying the same coordinate
fn merge_overlapped_vertex(
    world: &mut World,
    // mut cehm: Local<HashMap<IVec2, Entity>>,
) {
    // for every vertex v:
    // get from hashmap with IVec2 coord as key:
    //  get existing and merge into existing if existing is valid
    //  else put new into hashmap
    let mut cehm: HashMap<IVec2, Entity> = HashMap::new();
    let mut q =
        world.query_filtered::<(Entity, &GlobalTransform), (With<LineVertex>, Without<Preview>)>();
    let vertices: Box<[(Entity, IVec2)]> = q
        .iter(&world)
        .map(|x| (x.0, x.1.translation().truncate().as_ivec2()))
        .collect();
    for (this_vertex, c) in vertices.into_iter() {
        match cehm.insert(*c, *this_vertex) {
            Some(existing_vertex) => {
                // first, make branches referencing the old vertex reference the new vertex instead
                let mut existing_vertex_branches;
                {
                    let Some(eref) = world.get_entity(existing_vertex) else {
                        continue;
                    };
                    existing_vertex_branches = eref.get::<LineVertex>().unwrap().branches.clone();
                    for eseg in existing_vertex_branches.iter() {
                        let mut esegref = world.entity_mut(*eseg);
                        let mut seg = esegref.get_mut::<LineSegment>().unwrap();
                        if seg.a == existing_vertex {
                            seg.a = *this_vertex;
                        } else if seg.b == existing_vertex {
                            seg.b = *this_vertex;
                        } else {
                            panic!("misconnected line segment");
                        }
                    }
                }
                // update the branches on the new vertex
                world
                    .entity_mut(*this_vertex)
                    .get_mut::<LineVertex>()
                    .unwrap()
                    .branches
                    .append(&mut existing_vertex_branches);
                // delete the old vertex
                world.despawn(existing_vertex);
            }
            None => {}
        }
    }
}

/// this function iterates over all vertices and for each, bisects any segment that cross over it
fn bisect_merge(world: &mut World) {
    let mut qlv =
        world.query_filtered::<(Entity, &GlobalTransform), (With<LineVertex>, Without<Preview>)>();
    let mut qls = world.query_filtered::<(Entity, &LineSegment, &GlobalTransform, &SchematicElement), (With<LineSegment>, Without<Preview>)>();
    let vcoords: Box<[(Entity, Vec3)]> = qlv
        .iter(&world)
        .map(|(e, gt)| (e, gt.translation()))
        .collect();
    // bisection
    for c in vcoords.iter() {
        let mut ses = vec![];
        for (lse, seg, sgt, se) in qls.iter(&world) {
            if se.behavior.collides(
                &PickingCollider::Point(c.1.truncate()),
                sgt.compute_matrix().inverse(),
            ) {
                ses.push((
                    lse,
                    (seg.a, sgt.translation()),
                    (seg.b, sgt.transform_point(Vec3::new(1.0, 0.0, 0.0))),
                ));
            }
        }
        for (e, a, b) in ses {
            world.despawn(e);
            let ac = world
                .spawn(LineSegBundle::new(world.resource::<ElementsRes>(), a, *c))
                .id();
            let cb = world
                .spawn(LineSegBundle::new(world.resource::<ElementsRes>(), *c, b))
                .id();
            world
                .entity_mut(a.0)
                .get_mut::<LineVertex>()
                .unwrap()
                .branches
                .push(ac);
            world
                .entity_mut(b.0)
                .get_mut::<LineVertex>()
                .unwrap()
                .branches
                .push(cb);
            world
                .entity_mut(c.0)
                .get_mut::<LineVertex>()
                .unwrap()
                .branches
                .push(ac);
            world
                .entity_mut(c.0)
                .get_mut::<LineVertex>()
                .unwrap()
                .branches
                .push(cb);
        }
    }
    // merge
    let mut hs: HashSet<LineSegment> = HashSet::new();
    let mut qls: QueryState<(Entity, &LineSegment), _> =
        world.query_filtered::<(Entity, &LineSegment), Without<Preview>>();
    let els: Box<[(Entity, LineSegment)]> =
        qls.iter(&world).map(|(e, ls)| (e, (*ls).clone())).collect();
    for (e, ls) in els.into_iter() {
        match hs.insert(ls.clone()) {
            true => {
                // first insertion
            }
            false => {
                world.despawn(*e);
            }
        }
    }
}

/// system to prune line segs and vertices
/// deletes vertices with no branches or segments missing a vertex
/// needs to be exclusive system to fully complete in 1 frame
fn cull(world: &mut World) {
    let mut qls = world.query_filtered::<(Entity, &LineSegment), Without<Preview>>();
    let lses: Box<[(Entity, Entity, Entity)]> =
        qls.iter(&world).map(|x| (x.0, x.1.a, x.1.b)).collect();
    // delete segments missing one or both end point(s)
    for (eseg, a, b) in lses.iter() {
        if world.get_entity(*a).is_none() || world.get_entity(*b).is_none() {
            world.despawn(*eseg);
        }
    }
    // delete lonesome vertices
    let mut qlv = world.query::<(Entity, &mut LineVertex)>();
    let mut lves: Box<[(Entity, SmallVec<[Entity; 8]>)]> = qlv
        .iter(&world)
        .map(|x| (x.0, x.1.branches.clone()))
        .collect();
    for (e, ref mut lv) in lves.iter_mut() {
        *lv = lv
            .iter()
            .filter_map(|ls| {
                if world.get_entity(*ls).is_some() {
                    Some(*ls)
                } else {
                    None
                }
            })
            .collect();
        match lv.is_empty() {
            true => {
                world.despawn(*e);
            }
            false => {
                world
                    .entity_mut(*e)
                    .get_mut::<LineVertex>()
                    .unwrap()
                    .branches = lv.clone();
            }
        }
    }
}
