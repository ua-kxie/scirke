//! schematic elements
//! a schematic element may be reused in a circuit or device designer context (or more)
//! must support: picking by point/ray, by area intersect, by area contained
//! picking by point/ray should only ever mark 1 entity as picked
mod devices;
mod label;
mod netlisting;
mod nets;
mod readable_idgen;
mod spid;
mod spmanager;

pub use devices::{spawn_preview_device_from_type, DefaultDevices, DeviceParams};
pub use netlisting::SimAcHz;
pub use nets::{create_preview_lineseg, LineVertex};
pub use spid::{NetId, SpDeviceId};
pub use spmanager::SPRes;

use label::{sch_label_update, SchematicLabel};
use nets::{PickableLineSeg, PickableVertex};
use readable_idgen::IdTracker;
use spid::{SchType, SpDeviceType, SpType};
use spmanager::SPManagerPlugin;

use super::{
    infotext::InfoRes,
    material::SchematicMaterial,
    tools::{NewPickingCollider, PickingCollider, SelectEvt},
    SchematicChanged,
};

use bevy::{
    color::palettes::basic as basic_colors,
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
};

use euclid::default::{Box2D, Point2D};
use std::sync::Arc;

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
        debug!("deleting preview element");
        commands.entity(e).despawn();
    }
}

/// this systetm clears all preview marker compoenents from SchematicElements
pub fn persist_preview(
    commands: &mut Commands,
    q: &Query<Entity, (With<SchematicElement>, With<Preview>)>,
) {
    q.iter().for_each(|e| {
        commands.entity(e).remove::<Preview>();
    });
}

#[derive(Resource, Clone)]
pub struct ElementsRes {
    /// unit x line mesh, transformed by scale, rotation and translation to visualize a line segment
    pub mesh_unitx: Handle<Mesh>,
    /// circle mesh visualizing lineseg vertex
    pub mesh_dot: Handle<Mesh>,
    /// device port mesh
    pub mesh_port: Handle<Mesh>,

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
    pub pe_lineseg: PickableElement,
    /// lvse
    pub pe_linevertex: PickableElement,
    /// devices schematic element
    pub pe_device: PickableElement,
}

const MAT_SEL_COLOR: Srgba = basic_colors::YELLOW;
const MAT_PCK_COLOR: Srgba = basic_colors::WHITE;

impl FromWorld for ElementsRes {
    fn from_world(world: &mut World) -> Self {
        let wirecolor = basic_colors::AQUA.to_f32_array();

        // add port visuals
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let psize = 0.25;
        let mesh_port = meshes.add(
            Mesh::new(
                PrimitiveTopology::TriangleStrip,
                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
            )
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_POSITION,
                vec![
                    Vec3::new(-psize, psize, 0.0),
                    Vec3::new(-psize, -psize, 0.0),
                    Vec3::new(psize, psize, 0.0),
                    Vec3::new(psize, -psize, 0.0),
                ],
            )
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_COLOR,
                vec![basic_colors::RED.to_f32_array(); 4],
            )
            .with_inserted_indices(bevy::render::mesh::Indices::U32(
                (0..4).collect::<Vec<u32>>(),
            )),
        );

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
        let mut mats = world.resource_mut::<Assets<SchematicMaterial>>();
        ElementsRes {
            mesh_unitx,
            mesh_dot,
            mesh_port,

            mat_dflt: mats.add(SchematicMaterial {
                color: Color::BLACK.into(),
            }),
            mat_pckd: mats.add(SchematicMaterial {
                color: bevy::prelude::Color::Srgba(MAT_PCK_COLOR).into(),
            }),
            mat_seld: mats.add(SchematicMaterial {
                color: bevy::prelude::Color::Srgba(MAT_SEL_COLOR).into(),
            }),
            mat_alld: mats.add(SchematicMaterial {
                color: bevy::prelude::Color::Srgba(MAT_SEL_COLOR + MAT_PCK_COLOR).into(),
            }),

            pe_device: PickableElement {
                behavior: Arc::from(PickableDevice::_4x6()),
            },
            pe_lineseg: PickableElement {
                behavior: Arc::from(PickableLineSeg::default()),
            },
            pe_linevertex: PickableElement {
                behavior: Arc::from(PickableVertex::default()),
            },
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
pub struct PickableElement {
    behavior: Arc<dyn Pickable + Send + Sync + 'static>,
}

#[derive(Component, Clone, Reflect, Debug)]
#[reflect(Component)]
pub struct SchematicElement {
    schtype: SchType,
}

impl SchematicElement {
    pub fn get_schtype(&self) -> &SchType {
        &self.schtype
    }
    pub fn get_dtype(&self) -> Option<&SpDeviceType> {
        if let SchType::Spice(SpType::Device(dtyp)) = &self.schtype {
            Some(&dtyp)
        } else {
            None
        }
    }
}

/// Pickable trait to define how elements consider themselves "picked"
/// function needs sufficient information to determine collision
/// includes: picking collider, own hitbox
/// transform argument is the element transform's inverse (apply to cursor, see if it is over un-transformed element)
trait Pickable {
    fn collides(&self, pc: &PickingCollider, transform: Transform) -> bool;
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
/// The SystemSet for console/command related systems
pub enum ElectricalSet {
    /// systems directly responding to user input/events
    Direct,

    /// systems which react to a changed schematic
    React,

    /// prune set: wire pruning, netid assignment
    Prune,
}

pub struct ElementsPlugin;

impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (picking, selection, sch_label_update));
        app.add_systems(PostUpdate, set_mat);
        app.init_resource::<ElementsRes>();
        app.init_resource::<DefaultDevices>();
        app.register_type::<Selected>();
        app.register_type::<NetId>();
        app.register_type::<SchematicElement>();
        app.register_type::<SpDeviceType>();
        app.register_type::<SchType>();
        app.register_type::<SpType>();
        app.register_type::<SchematicLabel>();
        app.init_resource::<IdTracker>();
        app.add_plugins(devices::DevicesPlugin);
        app.add_plugins(nets::NetsPlugin);
        app.add_plugins(netlisting::NetlistPlugin);
        app.add_plugins(SPManagerPlugin);
        app.configure_sets(
            Update,
            (
                ElectricalSet::Direct.run_if(on_event::<SchematicChanged>()),
                ElectricalSet::React
                    .run_if(on_event::<SchematicChanged>())
                    .after(ElectricalSet::Direct),
                ElectricalSet::Prune
                    .run_if(on_event::<SchematicChanged>())
                    .after(ElectricalSet::React),
            ),
        );
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
    mut q_wse: Query<(Entity, &GlobalTransform, &PickableElement), Without<Preview>>,
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
    if !colliding.is_empty() && keys.just_pressed(KeyCode::KeyS) {
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
    q_valids: Query<Entity, With<PickableElement>>,
) {
    for selevt in e_sel.read().into_iter() {
        match selevt {
            SelectEvt::New => {
                sel_clear(&mut commands, &qes);
                sel_append(&mut commands, &qep);
            }
            SelectEvt::Append => sel_append(&mut commands, &qep),
            SelectEvt::Clear => sel_clear(&mut commands, &qes),
            SelectEvt::All => sel_all(&mut commands, &q_valids),
        }
    }
}

/// function to select all valid
fn sel_all(commands: &mut Commands, q_valid_sel: &Query<Entity, With<PickableElement>>) {
    let valids = q_valid_sel
        .iter()
        .map(|e| (e, Selected))
        .collect::<Vec<(Entity, Selected)>>();
    commands.insert_or_spawn_batch(valids.into_iter());
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
pub struct PickableDevice(Box2D<f32>);
impl PickableDevice {
    fn _4x6() -> Self {
        Self(Box2D::from_points([
            Point2D::new(-2.0, -3.0),
            Point2D::new(2.0, 3.0),
        ]))
    }
    fn _2x4() -> Self {
        Self(Box2D::from_points([
            Point2D::new(-1.0, -2.0),
            Point2D::new(1.0, 2.0),
        ]))
    }
}
impl Pickable for PickableDevice {
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
