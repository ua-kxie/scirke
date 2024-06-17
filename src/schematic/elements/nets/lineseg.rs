use super::{spid, ElementsRes, Pickable, PickableElement, SchematicElement};
use crate::schematic::{material::SchematicMaterial, tools::PickingCollider};
use bevy::{
    ecs::{
        entity::{Entity, EntityMapper, MapEntities},
        reflect::{ReflectComponent, ReflectMapEntities},
    },
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use euclid::default::{Box2D, Point2D};
use std::{hash::Hasher, sync::Arc};

const LINESEG_POINTS: [Vec3; 2] = [Vec3::splat(0.0), Vec3::new(1.0, 0.0, 0.0)];

/// LineSegment component containing references to defining ['LineVertex'] Entities
#[derive(Component, Reflect, Eq, Clone)]
#[reflect(Component, MapEntities)]
pub struct LineSegment {
    pub a: Entity,
    pub b: Entity,
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
    pub fn other_vertex(&self, this_vertex: Entity) -> Entity {
        if self.a == this_vertex {
            return self.b;
        } else {
            return self.a;
        }
    }
    pub fn get_a(&self) -> Entity {
        self.a
    }
    pub fn get_b(&self) -> Entity {
        self.b
    }
}

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

/// bundle defining a basic line segment
#[derive(Bundle)]
pub struct LineSegBundle {
    ls: LineSegment,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
    pe: PickableElement,
    se: SchematicElement,
}

impl LineSegBundle {
    pub fn new(eres: &ElementsRes, a: (Entity, Vec3), b: (Entity, Vec3)) -> Self {
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

/// this system updates the transforms of all linesegments so that its unitx mesh reflects the position of its defining vertices
pub fn transform_lineseg(
    gt: Query<&Transform, Without<LineSegment>>,
    mut lines: Query<(Entity, &LineSegment, &mut Transform, &mut Visibility)>,
    mut commands: Commands,
) {
    for (ent, ls, mut transform, mut visible) in lines.iter_mut() {
        let Ok(a) = gt.get(ls.get_a()) else {
            // a vertex cannot be found
            dbg!("4");
            commands.entity(ent).despawn();
            continue;
        };
        let Ok(b) = gt.get(ls.get_b()) else {
            // a vertex cannot be found
            dbg!("5");
            commands.entity(ent).despawn();
            continue;
        };
        // compute own transform to take unit X from (0, 0) -> (1, 0) to a -> b
        let m10 = b.translation - a.translation;
        let newt = Transform::from_translation(a.translation)
            .with_rotation(Quat::from_rotation_z(Vec2::X.angle_between(m10.truncate())))
            .with_scale(Vec3::splat(m10.length()));
        if newt.is_finite() {
            *visible = Visibility::Inherited;
            *transform = newt;
        } else {
            // dbg!("hiding infite lineseg", a, b);
            *visible = Visibility::Hidden;
        }
    }
}
