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

use std::{hash::Hasher, sync::Arc};

use super::{spid, ElementsRes, Pickable, PickableElement, Preview, SchematicElement};
use crate::schematic::{
    guides::ZoomInvariant, material::SchematicMaterial, tools::PickingCollider,
};
use bevy::{
    ecs::{
        entity::{Entity, EntityMapper, MapEntities},
        reflect::{ReflectComponent, ReflectMapEntities},
    },
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::smallvec::{smallvec, SmallVec},
};
use euclid::default::{Box2D, Point2D};
mod prune;
pub use prune::prune;
mod graph;
pub use graph::{connected_graphs, insert_netid};
/// LineSegment component containing references to defining ['LineVertex'] Entities
#[derive(Component, Reflect, Eq, Clone)]
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
impl std::hash::Hash for LineSegment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.a <= self.b {
            self.a.hash(state);
            self.b.hash(state);
        } else {
            self.b.hash(state);
            self.a.hash(state);
        }
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

const LINESEG_POINTS: [Vec3; 2] = [Vec3::splat(0.0), Vec3::new(1.0, 0.0, 0.0)];
/// A struct to define picking behavior specific to line segments
pub struct PickableLineSeg(Box2D<f32>);

impl Default for PickableLineSeg {
    fn default() -> Self {
        Self(Box2D::from_points([
            Point2D::splat(0.0),
            Point2D::new(1.0, 0.0),
        ]))
    }
}

impl Pickable for PickableLineSeg {
    fn collides(&self, pc: &PickingCollider, gt: Transform) -> bool {
        match pc {
            PickingCollider::Point(p) => {
                let t = gt.compute_matrix().inverse();
                if t.is_nan() {
                    return false;
                }
                let p1 = t.transform_point(p.extend(0.0));
                let (s, _, _) = t.to_scale_rotation_translation();
                self.0
                    // inflate proportional to inverse transform scale so that longer lines dont get bigger hit boxes
                    .inflate(-s.y * 0.5, s.y * 0.1)
                    .contains_inclusive(Point2D::new(p1.x, p1.y))
            }
            PickingCollider::AreaIntersect(pc) => {
                // true if pc intersects aabb of lineseg
                // TODO: improve this such that returns true if lineseg visually intersects pc
                pc.intersects(&Aabb2d::from_point_cloud(
                    Vec2::splat(0.0),
                    0.0,
                    &[
                        gt.transform_point(LINESEG_POINTS[0]).truncate(),
                        gt.transform_point(LINESEG_POINTS[1]).truncate(),
                    ],
                ))
            }
            PickingCollider::AreaContains(pc) => pc.contains(&Aabb2d::from_point_cloud(
                Vec2::splat(0.0),
                0.0,
                &[
                    gt.transform_point(LINESEG_POINTS[0]).truncate(),
                    gt.transform_point(LINESEG_POINTS[1]).truncate(),
                ],
            )),
        }
    }
}

/// defines the end points of a line
#[derive(Component, Clone, Reflect, Default)]
#[reflect(Component, MapEntities)]
pub struct LineVertex {
    pub branches: SmallVec<[Entity; 8]>, // anything above a three should be circuit schematic warning
}

impl MapEntities for LineVertex {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        for entity in &mut self.branches {
            *entity = entity_mapper.map_entity(*entity);
        }
    }
}

/// A struct defining picking behavior specific to line vertices
#[derive(Clone)]
pub struct PickableVertex(Box2D<f32>);
impl Default for PickableVertex {
    fn default() -> Self {
        Self(Box2D::from_points([
            Point2D::splat(-0.5),
            Point2D::splat(0.5),
        ]))
    }
}
impl Pickable for PickableVertex {
    fn collides(&self, pc: &PickingCollider, gt: Transform) -> bool {
        match pc {
            PickingCollider::Point(p) => {
                let t = gt.compute_matrix().inverse();
                if t.is_nan() {
                    return false;
                }
                let p1 = t.transform_point(p.extend(0.0));
                self.0.contains_inclusive(Point2D::new(p1.x, p1.y))
            }
            PickingCollider::AreaIntersect(pc) => pc.intersects(&Aabb2d::from_point_cloud(
                Vec2::splat(0.0),
                0.0,
                &[gt.transform_point(Vec3::splat(0.0)).truncate()],
            )),
            PickingCollider::AreaContains(pc) => pc.contains(&Aabb2d::from_point_cloud(
                Vec2::splat(0.0),
                0.0,
                &[gt.transform_point(Vec3::splat(0.0)).truncate()],
            )),
        }
    }
}

/// line vertex partial bundle (missing )
#[derive(Bundle)]
struct VertexBundle {
    vertex: LineVertex,
    pe: PickableElement,
    se: SchematicElement,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
    zi: ZoomInvariant,
}
impl VertexBundle {
    fn new(branch: Entity, eres: &Res<ElementsRes>, pt: Vec3) -> Self {
        Self {
            vertex: LineVertex {
                branches: smallvec![branch],
            },
            pe: eres.pe_linevertex.clone(),
            mat: MaterialMesh2dBundle {
                mesh: Mesh2dHandle(eres.mesh_dot.clone()),
                material: eres.mat_dflt.clone(),
                transform: Transform::from_translation(pt),
                ..Default::default()
            },
            zi: ZoomInvariant,
            se: SchematicElement {
                schtype: spid::SchType::Spice(spid::SpType::Net),
            },
        }
    }
}

/// bundle defining a basic line segment
#[derive(Bundle)]
struct LineSegBundle {
    ls: LineSegment,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
    pe: PickableElement,
    se: SchematicElement,
}

impl LineSegBundle {
    fn new(eres: &ElementsRes, a: (Entity, Vec3), b: (Entity, Vec3)) -> Self {
        let m10 = b.1 - a.1;
        let transform = Transform::from_translation(a.1)
            .with_rotation(Quat::from_rotation_z(Vec2::X.angle_between(m10.truncate())))
            .with_scale(Vec3::splat(m10.length()));

        let mat = MaterialMesh2dBundle {
            mesh: Mesh2dHandle(eres.mesh_unitx.clone()),
            material: eres.mat_dflt.clone(),
            transform: transform,
            ..Default::default()
        };
        let ls = LineSegment { a: a.0, b: b.0 };
        let pe = PickableElement {
            behavior: Arc::new(PickableLineSeg::default()),
        };
        Self {
            ls,
            mat,
            pe,
            se: SchematicElement {
                schtype: spid::SchType::Spice(spid::SpType::Net),
            },
        }
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
