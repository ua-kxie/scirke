//! schematic elements
//! a schematic element may be reused in a circuit or device designer context (or more)
//! must support: picking by point/ray, by area intersect, by area contained
//! picking by point/ray should only ever mark 1 entity as picked

use bevy::{
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
};

mod lineseg;
pub use lineseg::{create_preview_lineseg, lsse, lvse, LineSegment, LineVertex};

use super::{
    material::SchematicMaterial,
    tools::{NewPickingCollider, PickingCollider, SelectEvt},
};

/// marker component to mark entity as being previewed (constructed by an active tool)
/// entities marked [`SchematicElement`] but without this marker is persistent
#[derive(Component)]
pub struct Preview;

/// this systetm despawns all SchematicElements marked as Preview
pub fn despawn_preview(
    commands: &mut Commands,
    q: Query<Entity, (With<SchematicElement>, With<Preview>)>,
) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

/// this systetm clears all preview marker compoenents from SchematicElements
pub fn persist_preview(
    commands: &mut Commands,
    q: Query<Entity, (With<SchematicElement>, With<Preview>)>,
) {
    for e in q.iter() {
        commands.entity(e).remove::<Preview>();
    }
}

#[derive(Resource, Default)]
pub struct ElementsRes {
    /// unit x line mesh, transformed by scale, rotation and translation to visualize a line segment
    pub mesh_unitx: Option<Handle<Mesh>>,
    /// circle mesh visualizing lineseg vertex
    pub mesh_dot: Option<Handle<Mesh>>,

    /// default material
    pub mat_dflt: Option<Handle<SchematicMaterial>>,
    /// selected material
    pub mat_seld: Option<Handle<SchematicMaterial>>,
    /// picked material
    pub mat_pckd: Option<Handle<SchematicMaterial>>,
    /// selected + picked material
    pub mat_alld: Option<Handle<SchematicMaterial>>,
}

/// marker component to mark entity as colliding with picking collider
#[derive(Component)]
pub struct Picked;

/// marker component to mark entity as selected
#[derive(Component)]
pub struct Selected;

/// different components that impl a given trait T with functions to compute picking collision
/// add different

#[derive(Component)]
pub struct SchematicElement {
    behavior: Box<dyn Pickable + Send + Sync + 'static>,
}

/// Pickable trait to define how elements consider themselves "picked"
/// function needs sufficient information to determine collision
/// includes: picking collider, own hitbox
/// transform argument is the element transform's inverse (apply to cursor, see if it is over un-transformed element)
trait Pickable {
    fn collides(&self, pc: &PickingCollider, transform: Mat4) -> bool;
}

// entity wireseg schematicElement(TO)
// entity vertex schematicElement(TO)

pub struct ElementsPlugin;

impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (startup, lineseg::setup));
        app.add_systems(
            Update,
            (
                lineseg::transform_lineseg,
                picking,
                lineseg::prune,
                lineseg::extend_selection,
            ),
        );
        app.add_systems(PostUpdate, set_mat);
        app.init_resource::<ElementsRes>();
        app.register_type::<LineSegment>();
        app.register_type::<LineVertex>();
    }
}

/// startup system to initialize element resource
fn startup(
    mut eres: ResMut<ElementsRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<SchematicMaterial>>,
) {
    let c = Color::AQUAMARINE.rgba_linear_to_vec4();
    eres.mesh_unitx = Some(
        meshes.add(
            Mesh::new(
                PrimitiveTopology::LineList,
                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![Vec3::ZERO, Vec3::X])
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![c, c])
            .with_inserted_indices(bevy::render::mesh::Indices::U32(vec![0, 1])),
        ),
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
    eres.mesh_dot = Some(
        meshes.add(
            Mesh::new(
                PrimitiveTopology::TriangleStrip,
                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, hex)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![c; 6])
            .with_inserted_indices(bevy::render::mesh::Indices::U32(
                (0..6).collect::<Vec<u32>>(),
            )),
        ),
    );

    eres.mat_dflt = Some(mats.add(SchematicMaterial {
        color: Color::BLACK,
    }));
    eres.mat_pckd = Some(mats.add(SchematicMaterial {
        color: Color::WHITE,
    }));
    eres.mat_seld = Some(mats.add(SchematicMaterial {
        color: Color::YELLOW,
    }));
    eres.mat_alld = Some(mats.add(SchematicMaterial {
        color: Color::WHITE + Color::YELLOW,
    }));
}

/// system to apply selected/picked marker components
/// picking collision system:
/// on new picking collider:
/// get all schematic elements
/// check collision thorugh pickable trait obj
fn picking(
    mut commands: Commands,
    mut e_newpck: EventReader<NewPickingCollider>,
    mut q_wse: Query<(Entity, &GlobalTransform, &SchematicElement)>,
    mut e_sel: EventReader<SelectEvt>,

    qes: Query<Entity, With<Selected>>,
    qep: Query<Entity, With<Picked>>,
) {
    if let Some(NewPickingCollider(pc)) = e_newpck.read().last() {
        for (ent, sgt, se) in q_wse.iter_mut() {
            let t = sgt.compute_matrix().inverse();
            if t.is_nan() {
                continue;
            }
            match se.behavior.collides(pc, t) {
                true => {
                    commands.entity(ent).insert(Picked);
                }
                false => {
                    commands.entity(ent).remove::<Picked>();
                }
            }
        }
    };

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
            (None, None) => *mat = element_res.mat_dflt.clone().unwrap(),
            (None, Some(_)) => *mat = element_res.mat_seld.clone().unwrap(),
            (Some(_), None) => *mat = element_res.mat_pckd.clone().unwrap(),
            (Some(_), Some(_)) => *mat = element_res.mat_alld.clone().unwrap(),
        }
    }
}

// a line seg should be picked by area intersect if either vertex is contained
// a line seg should be picked by area contains if both vertex is contained
