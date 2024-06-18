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

use crate::schematic::{FreshLoad, SchematicChanged, SchematicLoaded};

use super::{spid, ElementsRes, Pickable, PickableElement, Preview, SchematicElement};
use bevy::{ecs::entity::Entity, prelude::*, sprite::Mesh2dHandle};
mod prune;
use lineseg::transform_lineseg;
use port::update_port_location;
pub use prune::prune;
mod graph;
pub use graph::{connected_graphs, insert_netid};
mod lineseg;
pub use lineseg::{LineSegBundle, LineSegment, PickableLineSeg};
mod linevertex;
pub use linevertex::{LineVertex, PickableVertex, VertexBundle};
mod port;
pub use port::{DevicePort, PortBundle, PortLabel};

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

pub struct NetsPlugin;

impl Plugin for NetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (transform_lineseg, update_port_location).run_if(on_event::<SchematicChanged>()),
        );
        app.add_systems(
            PostUpdate,
            (prune, insert_netid, connected_graphs)
                .chain()
                .run_if(on_event::<SchematicChanged>()),
        );
        app.add_systems(
            PreUpdate,
            insert_non_reflect.run_if(on_event::<SchematicLoaded>()),
        );
        app.register_type::<LineSegment>();
        app.register_type::<LineVertex>();
        app.register_type::<DevicePort>();
    }
}

/// this system iterates through
/// inserts non-refelct components for net type elements
/// useful for applying mesh handles and such after loading
fn insert_non_reflect(
    qv: Query<Entity, (With<FreshLoad>, With<LineVertex>, Without<DevicePort>)>,
    qs: Query<Entity, (With<FreshLoad>, With<LineSegment>)>,
    eres: Res<ElementsRes>,
    mut commands: Commands,
) {
    for lv in qv.iter() {
        let bundle = (
            eres.mat_dflt.clone(),
            Mesh2dHandle(eres.mesh_dot.clone()),
            eres.pe_linevertex.clone(),
        );
        commands.entity(lv).insert(bundle);
        commands.entity(lv).remove::<FreshLoad>();
    }
    for ls in qs.iter() {
        let bundle = (
            eres.mat_dflt.clone(),
            Mesh2dHandle(eres.mesh_unitx.clone()),
            eres.pe_lineseg.clone(),
        );
        commands.entity(ls).insert(bundle);
        commands.entity(ls).remove::<FreshLoad>();
    }
}
