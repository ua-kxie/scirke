//! Schematic Element defining a line
//! A Line element is comprised of 3 entities: vertex (x2) and lineseg (x1)
//! vertex can be shared between linesegs
//!
//! lines are planned to represent either nets in circuits or visual lines in device designer
//!
//! render performance: should use gpu instancing
//! bevy does this automatically for entities which share a mesh and material instance
//! hence, all line segments are rendered from the same unit-X mesh transformed to arbitrary location
//! all elements share a material instance, except those picked, selected, or both
//! (in which case all picked, selected, or both elements share a material instance)

use std::{
    collections::{HashMap, HashSet},
    f32::consts::PI,
};

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
use euclid::{
    approxeq::ApproxEq,
    default::{Box2D, Point2D},
};

/// LineSegment component containing references to defining ['LineVertex'] Entities
#[derive(Component, Reflect, Eq, Hash, Clone)]
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
impl PartialEq for LineSegment {
    fn eq(&self, other: &Self) -> bool {
        (self.a == other.a && self.b == other.b) || (self.a == other.b && self.b == other.a)
    }
}
impl LineSegment {
    /// returns vertex entity other than the one provided
    /// useful if one vertex of linesegment is known and want to find the other
    fn other_vertex(&self, this_vertex: Entity) -> Entity {
        if self.a == this_vertex {
            return self.b;
        } else {
            return self.a;
        }
    }
}

/// A struct to define picking behavior specific to line segments
#[derive(Default)]
struct PickableLineSeg;

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

/// defines the end points of a line
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

/// A struct defining picking behavior specific to line vertices
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

/// helper function to return SchematicElement containing lineseg picking behavior
pub fn lsse() -> SchematicElement {
    SchematicElement {
        behavior: Box::new(PickableLineSeg::default()),
    }
}

/// helper function to return SchematicElement containing vertex picking behavior
pub fn lvse() -> SchematicElement {
    SchematicElement {
        behavior: Box::new(PickableVertex::default()),
    }
}

/// line vertex partial bundle (missing )
#[derive(Bundle)]
struct VertexBundle {
    vertex: LineVertex,
    schematic_element: SchematicElement,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
    zi: ZoomInvariant,
}
impl VertexBundle {
    fn new(branch: Entity, eres: &Res<ElementsRes>, pt: Vec3) -> Self {
        Self {
            vertex: LineVertex {
                branches: smallvec![branch],
            },
            schematic_element: lvse(),
            mat: MaterialMesh2dBundle {
                mesh: Mesh2dHandle(eres.mesh_dot.clone().unwrap()),
                material: eres.mat_dflt.clone().unwrap(),
                transform: Transform::from_translation(pt),
                ..Default::default()
            },
            zi: ZoomInvariant,
        }
    }
}

/// bundle defining a basic line segment
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
    src_pt: Vec3,
    dst_pt: Vec3,
) {
    fn spawn_vertex(
        commands: &mut Commands,
        eres: &Res<ElementsRes>,
        lineseg_entity: Entity,
        vertex_entity: Entity,
        pt: Vec3,
    ) {
        commands
            .entity(vertex_entity)
            .insert((VertexBundle::new(lineseg_entity, eres, pt), Preview));
    }
    // vertex and segments have eachothers entity as reference
    // segment transform with scale zero since start and end are both at same point
    let src_entity = commands.spawn_empty().id();
    let dst_entity = commands.spawn_empty().id();
    let ls = LineSegBundle::new(eres, (src_entity, src_pt), (dst_entity, dst_pt));
    let lineseg_entity = commands.spawn((ls, Preview)).id();

    spawn_vertex(commands, eres, lineseg_entity, src_entity, src_pt);
    spawn_vertex(commands, eres, lineseg_entity, dst_entity, dst_pt);
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
    cull(world);
    merge_overlapped_vertex(world);

    bisect(world);
    // removing overlapping segments should always come after bisection
    // because bisection can produce overlapping segments
    cull_redundant_segments(world);
    combine_parallel(world);
}

/// this function merges vertices occupying the same coordinate
fn merge_overlapped_vertex(world: &mut World) {
    /// merges two vertices by appending the branches of `old` to `new`
    /// also goes through the branches of `old` and updates the references to `old` to `new`
    /// both `new` and `old` must be valid entities with [`LineVertex`] component
    fn merge_vertices(world: &mut World, new: Entity, old: Entity) {
        // get the branches of old
        let mut branches = world
            .entity(old)
            .get::<LineVertex>()
            .unwrap()
            .branches
            .clone();

        for &branch in branches.iter() {
            // if the branch is valid, update its references of `old` to `new`
            let Some(mut esegref) = world.get_entity_mut(branch) else {
                continue;
            };
            let mut seg = esegref.get_mut::<LineSegment>().unwrap();
            if seg.a == old {
                seg.a = new;
            } else if seg.b == old {
                seg.b = new;
            } else {
                panic!("misconnected line segment");
            }
        }

        // update the branches on the new vertex
        world
            .entity_mut(new)
            .get_mut::<LineVertex>()
            .unwrap()
            .branches
            .append(&mut branches);
        // delete the old vertex
        world.despawn(old);
    }

    // for every vertex v at coord:
    // get existing at coord and merge into existing if existing is valid
    // else put new into hashmap with coord as key
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
                merge_vertices(world, *this_vertex, existing_vertex);
            }
            None => {}
        }
    }
}

/// this function iterates over all vertices and for each, bisects any segment that cross over it
fn bisect(world: &mut World) {
    let mut qlv =
        world.query_filtered::<(Entity, &GlobalTransform), (With<LineVertex>, Without<Preview>)>();
    let mut qls = world.query_filtered::<(Entity, &LineSegment, &GlobalTransform, &SchematicElement), (With<LineSegment>, Without<Preview>)>();
    let vcoords: Box<[(Entity, Vec3)]> = qlv
        .iter(&world)
        .map(|(e, gt)| (e, gt.translation()))
        .collect();
    // bisection
    for vertex_coords in vcoords.iter() {
        let mut colliding_segments = vec![];
        for (lse, seg, sgt, se) in qls.iter(&world) {
            if se.behavior.collides(
                &PickingCollider::Point(vertex_coords.1.truncate()),
                sgt.compute_matrix().inverse(),
            ) {
                colliding_segments.push((
                    lse, // line segment entity
                    (seg.a, sgt.translation()), // vertex a entity and translation
                    (seg.b, sgt.transform_point(Vec3::new(1.0, 0.0, 0.0))), // vertex a entity and translation
                ));
            }
        }
        for (segment_entity, a, b) in colliding_segments {
            remove_lineseg(world, segment_entity);
            let ac = world
                .spawn(LineSegBundle::new(
                    world.resource::<ElementsRes>(),
                    a,
                    *vertex_coords,
                ))
                .id();
            let cb = world
                .spawn(LineSegBundle::new(
                    world.resource::<ElementsRes>(),
                    *vertex_coords,
                    b,
                ))
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
                .entity_mut(vertex_coords.0)
                .get_mut::<LineVertex>()
                .unwrap()
                .branches
                .push(ac);
            world
                .entity_mut(vertex_coords.0)
                .get_mut::<LineVertex>()
                .unwrap()
                .branches
                .push(cb);
        }
    }
}

fn combine_parallel(
    world: &mut World,
) {
    // remove vertices bisecting two parallel lines
    let mut qlv = world.query_filtered::<Entity, (With<LineVertex>, Without<Preview>)>();
    let all_vertices: Box<[Entity]> = qlv.iter(&world).collect();
    for vertex in all_vertices.iter() {
        merge_parallel(world, *vertex);
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
    let mut qlv = world.query_filtered::<Entity, (With<LineVertex>, Without<Preview>)>();
    let mut lves: Box<[Entity]> = qlv
        .iter(&world)
        .collect();
    for vertex_entity in lves.iter_mut() {
        let cleaned_branches: SmallVec<[Entity; 8]> = world.entity(*vertex_entity).get::<LineVertex>().unwrap().branches.iter()
            .filter_map(|ls| {
                if world.get_entity(*ls).is_some() {
                    Some(*ls)
                } else {
                    None
                }
            })
            .collect();
        if cleaned_branches.is_empty() {
            world.despawn(*vertex_entity);
        } else {
            world.entity_mut(*vertex_entity).get_mut::<LineVertex>().unwrap().branches = cleaned_branches;
        }
    }
}



/// removes line segments in world that share the same end points
/// for every removed line, go to vertices and remove references to self
fn cull_redundant_segments(world: &mut World) {
    // a LineSegment is eq if endpoints a and b are equal, or a == other.b and b == other.a
    // see PartialEq impl for [`LineSegment`]
    let mut hs: HashSet<LineSegment> = HashSet::new();
    let mut q_all_linesegs = world.query_filtered::<(Entity, &LineSegment), Without<Preview>>();
    let all_linesegs: Box<[(Entity, LineSegment)]> = q_all_linesegs
        .iter(&world)
        .map(|(e, ls)| (e, (*ls).clone()))
        .collect();
    for (segment_entity, ls) in all_linesegs.into_iter() {
        if !hs.insert(ls.clone()) {
           remove_lineseg(world, *segment_entity);
        }
    }
}

const ANGULAR_RADIANS_EPSILON: f32 = PI * 0.001;

// for every vertex:
// if exactly 2 branches and are parallel
// remove vertex and branches, connect other endpoint of each branch
fn merge_parallel(world: &mut World, vertex: Entity) {
    let branches = world
        .entity(vertex)
        .get::<LineVertex>()
        .unwrap()
        .branches
        .clone();
    if branches.len() == 2 {
        let gt0 = world.entity(branches[0]).get::<GlobalTransform>().unwrap();
        let gt1 = world.entity(branches[1]).get::<GlobalTransform>().unwrap();
        let rads_btwn = gt0
            .to_scale_rotation_translation()
            .1
            .normalize()
            .angle_between(gt1.to_scale_rotation_translation().1.normalize());
        if rads_btwn.approx_eq_eps(&PI, &ANGULAR_RADIANS_EPSILON)
            || rads_btwn.approx_eq_eps(&0.0, &ANGULAR_RADIANS_EPSILON)
        {
            // find endpoints
            let vertex0 = world
                .entity(branches[0])
                .get::<LineSegment>()
                .unwrap()
                .other_vertex(vertex);
            let vertex1 = world
                .entity(branches[1])
                .get::<LineSegment>()
                .unwrap()
                .other_vertex(vertex);
            // create new branch connecting
            add_lineseg(world, vertex0, vertex1);
            // despawn replaced
            world.despawn(branches[0]);
            world.despawn(branches[1]);
            world.despawn(vertex);
        }
    }
}

/// adds a branch connecting a and b
fn add_lineseg(
    world: &mut World,
    a: Entity,
    b: Entity,
) {
    // create lineseg bundle
    let lsb = LineSegBundle::new(
        world.resource::<ElementsRes>(),
        (
            a,
            world
                .entity(a)
                .get::<GlobalTransform>()
                .unwrap()
                .translation(),
        ),
        (
            b,
            world
                .entity(b)
                .get::<GlobalTransform>()
                .unwrap()
                .translation(),
        ),
    );
    let new_branch_id = world.spawn(lsb).id();
    world.entity_mut(a).get_mut::<LineVertex>().unwrap().branches.push(new_branch_id);
    world.entity_mut(b).get_mut::<LineVertex>().unwrap().branches.push(new_branch_id);
}

/// removes a lineseg from the world and also removes references to it in its end point vertices
fn remove_lineseg(
    world: &mut World,
    lineseg: Entity,
) {
    let ls = world.entity(lineseg).get::<LineSegment>().unwrap().clone();
    world.despawn(lineseg);
    world.entity_mut(ls.a).get_mut::<LineVertex>().unwrap().branches.retain(|x| *x != lineseg);
    world.entity_mut(ls.b).get_mut::<LineVertex>().unwrap().branches.retain(|x| *x != lineseg);
}