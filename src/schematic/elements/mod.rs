//! schematic elements
//! a schematic element may be reused in a circuit or device designer context (or more)
//! must support: picking by point/ray, by area intersect, by area contained
//! picking by point/ray should only ever mark 1 entity as picked

use bevy::{
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
};

mod lineseg;
pub use lineseg::create_lineseg;

use super::{
    material::WireMaterial,
    tools::{NewPickingCollider, PickingCollider},
};

#[derive(Resource, Default)]
pub struct ElementsRes {
    /// unit x line mesh, transformed by scale, rotation and translation to visualize a line segment
    pub unitx_mesh: Option<Handle<Mesh>>,
    /// default material
    pub mat_dflt: Option<Handle<WireMaterial>>,
    /// selected material
    pub mat_seld: Option<Handle<WireMaterial>>,
    /// picked material
    pub mat_pckd: Option<Handle<WireMaterial>>,
    /// selected + picked material
    pub mat_alld: Option<Handle<WireMaterial>>,
}

/// marker component to mark entity as colliding with picking collider
#[derive(Component)]
struct Picked;

/// marker component to mark entity as selected
#[derive(Component)]
struct Selected;

/// different components that impl a given trait T with functions to compute picking collision
/// add different

#[derive(Component)]
struct SchematicElement {
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
        app.add_systems(Update, (lineseg::transform_lineseg, picking));
        app.init_resource::<ElementsRes>();
    }
}

/// startup system to initialize element resource
fn startup(
    mut eres: ResMut<ElementsRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<WireMaterial>>,
) {
    let c = Color::AQUAMARINE.rgba_linear_to_vec4();
    eres.unitx_mesh = Some(
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
    eres.mat_dflt = Some(mats.add(WireMaterial {
        color: Color::BLACK,
    }));
    eres.mat_pckd = Some(mats.add(WireMaterial {
        color: Color::WHITE,
    }));
    eres.mat_seld = Some(mats.add(WireMaterial {
        color: Color::YELLOW,
    }));
    eres.mat_alld = Some(mats.add(WireMaterial {
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
    mut q_wse: Query<(
        Entity,
        &GlobalTransform,
        &SchematicElement,
        &mut Handle<WireMaterial>,
    )>,
    // q_se: Query<(&GlobalTransform, &SchematicElement, &Handle<SchematicMaterial>)>,
    element_res: Res<ElementsRes>,
) {
    let Some(NewPickingCollider(pc)) = e_newpck.read().last() else {
        return;
    };
    for (ent, sgt, se, mut wmat) in q_wse.iter_mut() {
        let t = sgt.compute_matrix().inverse();
        if t.is_nan() {
            continue;
        }
        if se.behavior.collides(pc, t) {
            commands.entity(ent).insert(Picked);
            *wmat = element_res.mat_pckd.clone().unwrap(); // TODO architecture needs to change once elements can be selected
        } else {
            *wmat = element_res.mat_dflt.clone().unwrap(); // TODO architecture needs to change once elements can be selected
        }
    }
}

// a line seg should be picked by area intersect if either vertex is contained
// a line seg should be picked by area contains if both vertex is contained
// need different code to run depending on collision target
