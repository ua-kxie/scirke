//! schematic elements
//! a schematic element may be reused in a circuit or device designer context (or more)
//! must support: picking by point/ray, by area intersect, by area contained
//! picking by point/ray should only ever mark 1 entity as picked

use std::sync::Arc;

use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
};
pub use devices::NewDevice;
mod devices;
mod nets;
pub use devices::Device;
use euclid::default::{Box2D, Point2D, Size2D};
use lyon_tessellation::{FillOptions, VertexBuffers};
pub use nets::{create_preview_lineseg, LineSegment, LineVertex};
use nets::{PickableLineSeg, PickableVertex};

use crate::bevyon::{self, build_mesh, fill, FillTessellator};

use super::{
    infotext::InfoRes,
    material::SchematicMaterial,
    tools::{NewPickingCollider, PickingCollider, SelectEvt},
    SchematicChanged,
};
pub use devices::DeviceType;
mod readable_idgen;
use readable_idgen::IdTracker;
mod spid;
pub use spid::SpId;
/// marker component to mark entity as being previewed (constructed by an active tool)
/// entities marked [`SchematicElement`] but without this marker is persistent
#[derive(Component)]
pub struct Preview;

/// this systetm despawns all SchematicElements marked as Preview
pub fn despawn_preview(
    commands: &mut Commands,
    q: &Query<Entity, (With<SchematicElement>, With<Preview>)>,
) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

/// this systetm clears all preview marker compoenents from SchematicElements
pub fn persist_preview(
    commands: &mut Commands,
    q: &Query<Entity, (With<SchematicElement>, With<Preview>)>,
) {
    for e in q.iter() {
        commands.entity(e).remove::<Preview>();
    }
}

#[derive(Resource)]
pub struct ElementsRes {
    /// unit x line mesh, transformed by scale, rotation and translation to visualize a line segment
    pub mesh_unitx: Handle<Mesh>,
    /// circle mesh visualizing lineseg vertex
    pub mesh_dot: Handle<Mesh>,
    /// resistor mesh
    pub mesh_res: Handle<Mesh>,

    /// default material
    pub mat_dflt: Handle<SchematicMaterial>,
    /// selected material
    pub mat_seld: Handle<SchematicMaterial>,
    /// picked material
    pub mat_pckd: Handle<SchematicMaterial>,
    /// selected + picked material
    pub mat_alld: Handle<SchematicMaterial>,

    /// schematic elements
    /// lsse
    pub se_lineseg: SchematicElement,
    /// lvse
    pub se_linevertex: SchematicElement,
    /// devices schematic element
    pub se_device: SchematicElement,

    /// resistor device type
    pub dtype_r: Handle<DeviceType>,
}

const MAT_SEL_COLOR: Color = Color::YELLOW;
const MAT_PCK_COLOR: Color = Color::WHITE;

impl FromWorld for ElementsRes {
    fn from_world(world: &mut World) -> Self {
        let wirecolor = Color::AQUAMARINE.rgba_linear_to_vec4();
        let devicecolor = Color::GREEN.rgba_linear_to_vec4();
        let mut fill_tess = world.resource_mut::<FillTessellator>();
        let mut path_builder = bevyon::path_builder();
        path_builder.add_rectangle(
            &Box2D {
                min: Point2D::new(-2.0, -3.0),
                max: Point2D::new(2.0, 3.0),
            },
            lyon_tessellation::path::Winding::Positive,
        );
        let path = path_builder.build();
        let mut buffers = VertexBuffers::new();
        fill(&mut *fill_tess, &path, &FillOptions::DEFAULT, &mut buffers);
        let mut res_mesh = build_mesh(&buffers).with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![devicecolor; buffers.vertices.len()],
        );
        // add port visuals
        let mut path_builder = bevyon::path_builder();
        let half_size = Size2D::splat(0.25);
        path_builder.add_rectangle(
            &Box2D::from_origin_and_size(
                Point2D::<f32>::new(0.0, 3.0) - half_size,
                half_size * 2.0,
            ),
            lyon_tessellation::path::Winding::Positive,
        );
        path_builder.add_rectangle(
            &Box2D::from_origin_and_size(
                Point2D::<f32>::new(0.0, -3.0) - half_size,
                half_size * 2.0,
            ),
            lyon_tessellation::path::Winding::Positive,
        );
        let path = path_builder.build();
        let mut buffers = VertexBuffers::new();
        fill(&mut *fill_tess, &path, &FillOptions::DEFAULT, &mut buffers);
        let ports_mesh = build_mesh(&buffers).with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![Color::RED.rgba_linear_to_vec4(); buffers.vertices.len()],
        );
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        res_mesh.merge(ports_mesh);
        let mesh_res = meshes.add(res_mesh);

        let mesh_unitx = meshes.add(
            Mesh::new(
                PrimitiveTopology::LineList,
                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![Vec3::ZERO, Vec3::X])
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![wirecolor; 2])
            .with_inserted_indices(bevy::render::mesh::Indices::U32(vec![0, 1])),
        );
        let hex = [
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(-0.866025, 0.5, 0.0),
            Vec3::new(0.866025, 0.5, 0.0),
            Vec3::new(-0.866025, -0.5, 0.0),
            Vec3::new(0.866025, -0.5, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
        ]
        .into_iter()
        .map(|x| x * 2.0)
        .collect::<Vec<Vec3>>();
        let mesh_dot = meshes.add(
            Mesh::new(
                PrimitiveTopology::TriangleStrip,
                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, hex)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![wirecolor; 6])
            .with_inserted_indices(bevy::render::mesh::Indices::U32(
                (0..6).collect::<Vec<u32>>(),
            )),
        );
        let mut dtypes = world.resource_mut::<Assets<DeviceType>>();
        let dtype_r = dtypes.add(devices::DeviceType::new_resistor());
        let mut mats = world.resource_mut::<Assets<SchematicMaterial>>();
        ElementsRes {
            mesh_unitx,
            mesh_dot,
            mesh_res,

            mat_dflt: mats.add(SchematicMaterial {
                color: Color::BLACK,
            }),
            mat_pckd: mats.add(SchematicMaterial {
                color: MAT_PCK_COLOR,
            }),
            mat_seld: mats.add(SchematicMaterial {
                color: MAT_SEL_COLOR,
            }),
            mat_alld: mats.add(SchematicMaterial {
                color: MAT_SEL_COLOR + MAT_PCK_COLOR,
            }),

            se_device: SchematicElement {
                behavior: Arc::from(PickableRes::default()),
            },
            se_lineseg: SchematicElement {
                behavior: Arc::from(PickableLineSeg::default()),
            },
            se_linevertex: SchematicElement {
                behavior: Arc::from(PickableVertex::default()),
            },

            dtype_r,
        }
    }
}

/// marker component to mark entity as colliding with picking collider
#[derive(Component)]
pub struct Picked;

/// marker component to mark entity as selected
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Selected;

/// different components that impl a given trait T with functions to compute picking collision
#[derive(Component, Clone)]
pub struct SchematicElement {
    behavior: Arc<dyn Pickable + Send + Sync + 'static>,
}

/// Pickable trait to define how elements consider themselves "picked"
/// function needs sufficient information to determine collision
/// includes: picking collider, own hitbox
/// transform argument is the element transform's inverse (apply to cursor, see if it is over un-transformed element)
trait Pickable {
    fn collides(&self, pc: &PickingCollider, transform: Transform) -> bool;
}

pub struct ElementsPlugin;

impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<DeviceType>();
        // app.add_systems(Startup, startup);
        app.add_systems(
            Update,
            (
                nets::transform_lineseg,
                picking,
                selection,
                devices::add_preview_device,
            ),
        );
        app.add_systems(PostUpdate, set_mat);
        app.add_systems(
            PostUpdate,
            (nets::prune, nets::insert_spid, nets::connected_graphs)
                .chain()
                .run_if(on_event::<SchematicChanged>()),
        );
        app.init_resource::<ElementsRes>();
        app.register_type::<LineSegment>();
        app.register_type::<LineVertex>();
        app.register_type::<Selected>();
        app.register_type::<Device>();
        app.init_resource::<IdTracker>();
        app.add_event::<NewDevice>();
    }
}

/// system to apply selected/picked marker components
/// picking collision system:
/// on new picking collider:
/// get all schematic elements
/// check collision thorugh pickable trait obj
fn picking(
    mut commands: Commands,
    mut e_newpck: EventReader<NewPickingCollider>,
    mut q_wse: Query<(Entity, &GlobalTransform, &SchematicElement), Without<Preview>>,
    mut colliding: Local<Vec<Entity>>,
    mut idx: Local<usize>,
    keys: Res<ButtonInput<KeyCode>>,
    mut infores: ResMut<InfoRes>,
) {
    // colliding: vector of colliding entities under current point picking collider
    // should be empty if picking collider is not a point
    // idx: current index of colliding, such that collding[idx] is the picked element
    // process any new picking collider event
    if let Some(NewPickingCollider(pc)) = e_newpck.read().last() {
        infores.set_picked(None);
        if let PickingCollider::Point(_) = pc {
            // update `colliding`, and unset any Picked
            colliding.clear();
            for (ent, sgt, se) in q_wse.iter_mut() {
                commands.entity(ent).remove::<Picked>();
                if se.behavior.collides(pc, sgt.compute_transform()) {
                    colliding.push(ent);
                }
            }
            // set one, if any, entity as picked
            if let Some(&ent) = colliding.first() {
                infores.set_picked(Some(ent));
                commands.entity(ent).insert(Picked);
                *idx = 0;
            }
        } else {
            // area selection: reset Picked, then mark any colliding as Picked
            colliding.clear();
            for (ent, sgt, se) in q_wse.iter_mut() {
                if se.behavior.collides(pc, sgt.compute_transform()) {
                    commands.entity(ent).insert(Picked);
                } else {
                    commands.entity(ent).remove::<Picked>();
                }
            }
        }
    }
    // if cycle command is pressed and colliding vector is not empty
    if !colliding.is_empty() && keys.just_pressed(KeyCode::KeyC) {
        commands.entity(colliding[*idx]).remove::<Picked>();
        *idx = (*idx + 1) % colliding.len();
        commands.entity(colliding[*idx]).insert(Picked);
    }
}

fn selection(
    mut commands: Commands,
    mut e_sel: EventReader<SelectEvt>,
    qes: Query<Entity, With<Selected>>,
    qep: Query<Entity, With<Picked>>,
) {
    for selevt in e_sel.read().into_iter() {
        match selevt {
            SelectEvt::New => {
                sel_clear(&mut commands, &qes);
                sel_append(&mut commands, &qep);
            }
            SelectEvt::Append => sel_append(&mut commands, &qep),
            SelectEvt::Clear => sel_clear(&mut commands, &qes),
        }
    }
}

/// function to clear selected marker from all elements
fn sel_clear(commands: &mut Commands, qes: &Query<Entity, With<Selected>>) {
    for e in qes.iter() {
        commands.entity(e).remove::<Selected>();
    }
}

/// function to mark as selected all elements already marked as picked
fn sel_append(command: &mut Commands, qep: &Query<Entity, With<Picked>>) {
    for e in qep.iter() {
        command.entity(e).insert(Selected);
    }
}

/// this sets the material of elements to visualize which are picked and/or selected
fn set_mat(
    mut q_sse: Query<(
        &mut Handle<SchematicMaterial>,
        Option<&Picked>,
        Option<&Selected>,
    )>,
    element_res: Res<ElementsRes>,
) {
    for (mut mat, pcked, seld) in q_sse.iter_mut() {
        match (pcked, seld) {
            (None, None) => *mat = element_res.mat_dflt.clone(),
            (None, Some(_)) => *mat = element_res.mat_seld.clone(),
            (Some(_), None) => *mat = element_res.mat_pckd.clone(),
            (Some(_), Some(_)) => *mat = element_res.mat_alld.clone(),
        }
    }
}

/// A struct defining picking behavior for Aabbs
#[derive(Clone)]
pub struct PickableRes(Box2D<f32>);
impl Default for PickableRes {
    fn default() -> Self {
        Self(Box2D::from_points([
            Point2D::new(-2.0, -3.0),
            Point2D::new(2.0, 3.0),
        ]))
    }
}
impl Pickable for PickableRes {
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
                gt.transform_point(Vec3::splat(0.0)).truncate(),
                0.0,
                &[
                    Vec2::new(self.0.min.x, self.0.min.y),
                    Vec2::new(self.0.max.x, self.0.max.y),
                ],
            )),
            PickingCollider::AreaContains(pc) => pc.contains(&Aabb2d::from_point_cloud(
                gt.transform_point(Vec3::splat(0.0)).truncate(),
                0.0,
                &[
                    Vec2::new(self.0.min.x, self.0.min.y),
                    Vec2::new(self.0.max.x, self.0.max.y),
                ],
            )),
        }
    }
}
