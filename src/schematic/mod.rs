

use bevy::{ecs::entity::EntityHashMap, scene::{ron, serde::SceneDeserializer}};
use serde::de::DeserializeSeed;

use self::{
    camera::CameraPlugin, elements::ElementsPlugin, guides::GuidesPlugin, infotext::InfoPlugin,
    material::SchematicMaterial, tools::ToolsPlugin,
};
use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
    sprite::{Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};
use elements::{ElementsRes, LineSegment, LineVertex, SchematicElement};

mod camera;
mod elements;
mod guides;
mod infotext;
mod material;
mod tools;

// Snapped marker component: system to goes around snapping transform of such entities
#[derive(Component)]
pub struct Snap {
    /// snap step size, coords are snapped as:
    /// (coord/step).round() * step
    pub world_step: f32,
    // pub space: Space,
}

impl Snap {
    const DEFAULT: Self = Snap { world_step: 1.0 };
}

pub enum Space {
    World,
    Clip,
}

/// [`SystemSet`] for system which performs snapping.
/// Resides in [`PostUpdate`] schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct SnapSet;

pub struct SchematicPlugin;

impl Plugin for SchematicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CameraPlugin,
            InfoPlugin,
            GuidesPlugin,
            ElementsPlugin,
            ToolsPlugin,
        ));
        app.configure_sets(
            PostUpdate,
            SnapSet.before(bevy::transform::TransformSystem::TransformPropagate),
        );
        // app.add_systems(Startup, setup);
        app.add_systems(PostUpdate, snap.in_set(SnapSet));
        app.add_systems(Update, save.run_if(input_just_pressed(KeyCode::KeyS)));
        app.add_systems(Update, load.run_if(input_just_pressed(KeyCode::KeyL)));
        app.add_plugins(Material2dPlugin::<SchematicMaterial>::default());
    }
}

/// this system snaps all applicable entities
fn snap(mut e: Query<(&mut Transform, &Snap), Changed<GlobalTransform>>) {
    for (mut t, s) in e.iter_mut() {
        t.translation = (t.translation / s.world_step).round() * s.world_step;
    }
}

/// helper system to test things quick and dirty
fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<SchematicMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mat_bundle = MaterialMesh2dBundle {
        mesh: Mesh2dHandle(
            meshes.add(
                Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD)
                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![Vec3::ZERO, Vec3::X])
                    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![Vec4::ONE, Vec4::ONE])
                    .with_inserted_indices(bevy::render::mesh::Indices::U32(vec![0, 1])),
            ),
        ),
        material: materials.add(SchematicMaterial {
            color: Color::BLACK.with_a(0.0),
        }),
        transform: Transform::from_scale(Vec3::splat(1.0)),
        ..Default::default()
    };
    commands.spawn(mat_bundle);
}

fn save(
    world: &mut World,
    // keys: Res<ButtonInput<KeyCode>>,
) {
    let mut binding = world.query_filtered::<Entity, With<SchematicElement>>();
    let ents = binding.iter(world);
    let dsb = DynamicSceneBuilder::from_world(world)
        .deny::<Mesh2dHandle>()
        .deny::<Handle<SchematicMaterial>>()
        .allow::<elements::LineVertex>()
        .allow::<elements::LineSegment>()
        .extract_entities(ents);
    let reg = world.resource::<AppTypeRegistry>().clone();

    let data;
    let a = dsb.build();
    for e in &a.entities {
        dbg!(e.entity);
        for c in &e.components {
            dbg!(c);
        }
    }
    match a.serialize_ron(&reg) {
        Ok(data1) => {
            data = data1;
        }
        Err(err) => {
            eprintln!("Application error: {err}");
            return;
        }
    }
    std::fs::write("out/foo1.ron", data).expect("Unable to write file");
}

fn load(
    world: &mut World,
) {
    // cant seem to get loading through assetserver as shown in bevy example to work
    // looking a this instead
    // https://github.com/UmbraLuminosa/Proof-of-Concept-Editor-in-Bevy/blob/main/src/ui_plugin/undo_plugin.rs
    // let type_registry = world.resource::<AppTypeRegistry>().clone();
    // let scene = world.resource::<AssetServer>().load("../out/foo.ron");
    // world.spawn(DynamicSceneBundle { scene, ..default() });
    let s = std::fs::read_to_string("out/foo.ron").unwrap();
    let mut deserializer = ron::de::Deserializer::from_str(&s).unwrap();
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let scene_deserializer = SceneDeserializer {
        type_registry: &type_registry.read(),
    };
    let result = scene_deserializer.deserialize(&mut deserializer).unwrap();
    // for e in &result.entities {
    //     dbg!(e.entity);
    //     for c in &e.components {
    //         dbg!(c);
    //     }
    // }
    let mut entity_map: EntityHashMap<Entity> = EntityHashMap::default();
    if let Err(e) = result.write_to_world_with(world, &mut entity_map, &type_registry) {
        println!("Error updating world: {}", e);
    }

    let mut q = world.query_filtered::<Entity, With<LineVertex>>();
    let es = q.iter(&world).collect::<Vec<Entity>>();
    let lv_meshes = vec![
        (
            Mesh2dHandle(world.resource::<ElementsRes>().mesh_dot.clone().unwrap()),
            world.resource::<ElementsRes>().mat_dflt.clone().unwrap()
        );
        es.len()
    ];
    let elvs = es.into_iter().zip(lv_meshes);

    let mut q = world.query_filtered::<Entity, With<LineSegment>>();
    let es = q.iter(&world).collect::<Vec<Entity>>();
    let ls_meshes: Vec<(_, Handle<SchematicMaterial>)> = vec![
        (
            Mesh2dHandle(world.resource::<ElementsRes>().mesh_unitx.clone().unwrap()),
            world.resource::<ElementsRes>().mat_dflt.clone().unwrap()
        );
        es.len()
    ];
    let elss = es.into_iter().zip(ls_meshes);
    let _ = world.insert_or_spawn_batch(elvs.chain(elss));
}

// fn load(
//     mut commands: Commands, asset_server: Res<AssetServer>,
//     q: Query<Entity, With<LineVertex>>,
//     q1: Query<Entity, With<LineSegment>>,
//     r: Res<ElementsRes>,
// ) {
//     // "Spawning" a scene bundle creates a new entity and spawns new instances
//     // of the given scene's entities as children of that entity.
//     commands.spawn(DynamicSceneBundle {
//         // Scenes are loaded just like any other asset.
//         scene: asset_server.load("../out/foo.ron"),
//         ..default()
//     });
    
//     let es = q.iter().collect::<Vec<Entity>>();
//     let lv_meshes = vec![
//         (
//             Mesh2dHandle(r.mesh_dot.clone().unwrap()),
//             r.mat_dflt.clone().unwrap()
//         );
//         es.len()
//     ];
//     let elvs = es.into_iter().zip(lv_meshes);

//     let es = q1.iter().collect::<Vec<Entity>>();
//     let ls_meshes = vec![
//         (
//             Mesh2dHandle(r.mesh_unitx.clone().unwrap()),
//             r.mat_dflt.clone().unwrap()
//         );
//         es.len()
//     ];
//     let elss = es.into_iter().zip(ls_meshes);
//     dbg!(&elss);
//     commands.insert_or_spawn_batch(elvs.chain(elss));
// }