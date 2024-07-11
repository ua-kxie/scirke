use super::{spid, ElementsRes, Pickable, PickableElement, SchematicElement};
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
};
use smallvec::{smallvec, SmallVec};
use euclid::default::{Box2D, Point2D};

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
pub struct VertexBundle {
    vertex: LineVertex,
    pe: PickableElement,
    se: SchematicElement,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
    zi: ZoomInvariant,
}
impl VertexBundle {
    pub fn new(branch: Entity, eres: &Res<ElementsRes>, pt: Vec3) -> Self {
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
