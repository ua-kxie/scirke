use std::{
    collections::{HashMap, HashSet},
    f32::consts::PI,
};

use super::{ElementsRes, LineSegBundle, LineSegment, LineVertex, PickableElement, Preview};
use crate::schematic::tools::PickingCollider;
use bevy::{ecs::entity::Entity, prelude::*, utils::smallvec::SmallVec};
use euclid::approxeq::ApproxEq;

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
    // delete orphaned vertices or incomplete segments
    cull(world);
    //
    merge_overlapped_vertex(world);
    // bisect segments at vertices
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
        world.query_filtered::<(Entity, &Transform), (With<LineVertex>, Without<Preview>)>();
    let vertices: Box<[(Entity, IVec2)]> = q
        .iter(&world)
        .map(|x| (x.0, x.1.translation.truncate().as_ivec2()))
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
        world.query_filtered::<(Entity, &Transform), (With<LineVertex>, Without<Preview>)>();
    let mut qls = world.query_filtered::<(Entity, &LineSegment, &Transform, &PickableElement), (With<LineSegment>, Without<Preview>)>();
    let vcoords: Box<[(Entity, Vec3)]> = qlv
        .iter(&world)
        .map(|(e, gt)| (e, gt.translation))
        .collect();
    // bisection
    for (this_v_entity, this_v_coords) in vcoords.iter() {
        let mut colliding_segments = vec![];
        // collect colliding segments
        for (lse, seg, sgt, pe) in qls.iter(&world) {
            if pe
                .behavior
                .collides(&PickingCollider::Point(this_v_coords.truncate()), *sgt)
            {
                colliding_segments.push((
                    lse,   // line segment entity
                    seg.a, // vertex a entity
                    seg.b, // vertex b entity
                ));
            }
        }
        // for all collding segments
        for (segment_entity, a, b) in colliding_segments {
            remove_lineseg(world, segment_entity);
            add_lineseg(world, a, *this_v_entity);
            add_lineseg(world, *this_v_entity, b);
        }
    }
}

fn combine_parallel(world: &mut World) {
    // remove vertices bisecting two parallel lines
    let mut qlv = world.query_filtered::<Entity, (With<LineVertex>, Without<Preview>)>();
    let all_vertices: Box<[Entity]> = qlv.iter(&world).collect();
    for vertex in all_vertices.iter() {
        merge_parallel(world, *vertex);
    }
}

/// system to prune line segs and vertices
/// deletes vertices with no branches or segments missing a vertex
/// needs to be exclusive system to fully complete in 1 frame?
fn cull(world: &mut World) {
    let mut qls = world.query_filtered::<(Entity, &LineSegment), Without<Preview>>();
    let lses: Box<[(Entity, Entity, Entity)]> =
        qls.iter(&world).map(|x| (x.0, x.1.a, x.1.b)).collect();
    // delete segments missing one or both end point(s)
    for (eseg, a, b) in lses.iter() {
        if world.get_entity(*a).is_none() || world.get_entity(*b).is_none() {
            remove_lineseg(world, *eseg);
            // world.despawn(*eseg);
        }
    }
    // delete lonesome vertices
    let mut qlv = world.query_filtered::<Entity, (With<LineVertex>, Without<Preview>)>();
    let mut lves: Box<[Entity]> = qlv.iter(&world).collect();
    for vertex_entity in lves.iter_mut() {
        let cleaned_branches: SmallVec<[Entity; 8]> = world
            .entity(*vertex_entity)
            .get::<LineVertex>()
            .unwrap()
            .branches
            .iter()
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
            world
                .entity_mut(*vertex_entity)
                .get_mut::<LineVertex>()
                .unwrap()
                .branches = cleaned_branches;
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
        // look at transform here because global transform has not yet been updated
        let gt0 = world.entity(branches[0]).get::<Transform>().unwrap();
        let gt1 = world.entity(branches[1]).get::<Transform>().unwrap();
        let rads_btwn = gt0
            .rotation
            .normalize()
            .angle_between(gt1.rotation.normalize());
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
            remove_lineseg(world, branches[0]);
            remove_lineseg(world, branches[1]);
            world.despawn(vertex); // todo: this is a problem if one of the linesegs has both ends connected to same vertex
        }
    }
}

/// adds a branch connecting a and b
fn add_lineseg(world: &mut World, a: Entity, b: Entity) {
    // create lineseg bundle
    let lsb = LineSegBundle::new(
        world.resource::<ElementsRes>(),
        (a, world.entity(a).get::<Transform>().unwrap().translation),
        (b, world.entity(b).get::<Transform>().unwrap().translation),
    );
    let new_branch_id = world.spawn(lsb).id();
    world
        .entity_mut(a)
        .get_mut::<LineVertex>()
        .unwrap()
        .branches
        .push(new_branch_id);
    world
        .entity_mut(b)
        .get_mut::<LineVertex>()
        .unwrap()
        .branches
        .push(new_branch_id);
}

/// removes a lineseg from the world and also removes references to it in its end point vertices
fn remove_lineseg(world: &mut World, lineseg: Entity) {
    let ls = world.entity(lineseg).get::<LineSegment>().unwrap().clone();
    world.despawn(lineseg);
    world.get_entity_mut(ls.a).map(|mut x| {
        x.get_mut::<LineVertex>()
            .unwrap()
            .branches
            .retain(|x| *x != lineseg);
    });
    world.get_entity_mut(ls.b).map(|mut x| {
        x.get_mut::<LineVertex>()
            .unwrap()
            .branches
            .retain(|x| *x != lineseg);
    });
}
