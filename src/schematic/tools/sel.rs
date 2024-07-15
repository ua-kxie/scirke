//! idle/selection tool
//! handles objection picking and listens for user input for entering another tool

use super::{transform::TransformType, MergeLoadEvent, SchematicToolState};
use crate::{
    bevyon::{self, CompositeMeshData, SubMesh, TessInData},
    schematic::{
        camera::SchematicCamera,
        electrical::{PickableElement, Preview, Selected},
        guides::{NewSnappedCursorPos, SchematicCursor},
        material::SchematicMaterial,
        tools::ToolsPreviewPipeline,
        EntityLoadSet, LoadEvent, SchematicChanged,
    },
};
use bevy::{
    color::palettes::basic as basic_colors,
    input::{keyboard::KeyboardInput, ButtonState},
    math::bounding::Aabb2d,
    prelude::*,
    sprite::MaterialMesh2dBundle,
};
use bevy_save::WorldSaveableExt;

/// struct describes the picking collider
pub enum PickingCollider {
    Point(Vec2),
    AreaIntersect(Aabb2d),
    AreaContains(Aabb2d),
}

/// event to be sent when elements marked picked should be marked with selected
#[derive(Event)]
pub enum SelectEvt {
    New,    // replace selection with current picked set
    Append, // append to selection with current picked set
    // Subtract, // remove from selection the current picked set
    // Inverse,  // invert selected entities
    Clear, // deselect all
    All,   // select all valid targets
}

/// event to be sent when the picking collider changes
#[derive(Event)]
pub struct NewPickingCollider(pub PickingCollider);

impl NewPickingCollider {
    pub fn point(p: Vec2) -> NewPickingCollider {
        NewPickingCollider(PickingCollider::Point(p))
    }
    pub fn min_max(min: Vec2, max: Vec2) -> NewPickingCollider {
        let a = max - min;
        match (a.x * a.y).is_sign_positive() {
            true => NewPickingCollider(PickingCollider::AreaContains(Aabb2d::from_point_cloud(
                Vec2::splat(0.0),
                0.0,
                &[min, max],
            ))),
            false => NewPickingCollider(PickingCollider::AreaIntersect(Aabb2d::from_point_cloud(
                Vec2::splat(0.0),
                0.0,
                &[min, max],
            ))),
        }
    }
}
#[derive(Component)]
struct SelMarker;

#[derive(Bundle)]
struct SelToolMarkerBundle {
    marker: SelMarker,
    tess_data: CompositeMeshData,
    mat_bundle: MaterialMesh2dBundle<SchematicMaterial>,
}

/// Selection tool resources
/// stores the coordinate where left mouse button was clicked down at
#[derive(Resource, Default)]
struct SelToolRes {
    sel_area_origin: Vec2,
}

pub struct SelToolPlugin;

impl Plugin for SelToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            PreUpdate,
            (main, listener).run_if(in_state(SchematicToolState::Idle)),
        );
        app.init_resource::<SelToolRes>();
        app.add_event::<NewPickingCollider>();
        app.add_event::<SelectEvt>();
        app.add_systems(
            PreUpdate,
            (
                tools_select
                    .in_set(EntityLoadSet::Direct)
                    .run_if(in_state(SchematicToolState::Idle)),
                (save_load, post_serde)
                    .chain()
                    .run_if(on_event::<MergeLoadEvent>()),
            )
                .chain(),
        );
    }
}

/// this system listens to user inputs valid during idle
fn listener(
    keys: Res<ButtonInput<KeyCode>>,
    qc: Query<Entity, With<Selected>>,
    mut commands: Commands,
    mut e_schchanged: EventWriter<SchematicChanged>,
) {
    if keys.just_released(KeyCode::Delete) {
        for e in qc.iter() {
            debug!("deleting selected entity");
            commands.entity(e).despawn();
        }
        e_schchanged.send(SchematicChanged);
    }
}

fn post_serde(
    mut commands: Commands,
    qc: Query<Entity, With<SchematicCursor>>,
    q_unpicked: Query<Entity, (Without<Selected>, With<Preview>, With<PickableElement>)>,
    mut q_transform: Query<(&GlobalTransform, &mut Transform)>,
    q_scparent: Query<(&GlobalTransform, &Children), With<SchematicCursor>>,
    mut ev_sch_changed: EventWriter<SchematicChanged>,
) {
    let cursor = qc.single();
    // despawn Preview, Pickable, Without<Selected>
    commands
        .entity(cursor)
        .remove_children(&q_unpicked.iter().collect::<Box<[Entity]>>());
    for e in q_unpicked.iter() {
        debug!("deletied entity: unpicked, selected");
        commands.entity(e).despawn();
    }
    // anything Preview gets added to schematiccursor children, transform adjusted
    let (cursor_gt, cchildren) = q_scparent.single();
    let offset = cursor_gt.translation();
    let mut children = vec![];
    for c in cchildren {
        children.push(*c);
    }
    for c in children {
        let (gt, mut t) = q_transform.get_mut(c).unwrap();
        (*t).translation = gt.translation() - offset;
    }
    ev_sch_changed.send(SchematicChanged); // TODO port gets double dipped between transform propagate and port location update
}

fn save_load(world: &mut World) {
    debug!("save-load");
    // everything in schematic gets saved
    world
        .save(ToolsPreviewPipeline)
        .expect("Failed to save copy");
    // copy of everything gets loaded with Preview tag
    world
        .load(ToolsPreviewPipeline)
        .expect("Failed to load copy");
}
const WIRE_TOOL_KEY: KeyCode = KeyCode::KeyW;
const DEVICE_SPAWN_TOOL_KEY: KeyCode = KeyCode::KeyD;
const MOVE_KEY: KeyCode = KeyCode::KeyM;
const COPY_KEY: KeyCode = KeyCode::KeyC;
fn tools_select(
    commands: Commands,
    mut evt_keys: EventReader<KeyboardInput>,
    mut toolst_next: ResMut<NextState<SchematicToolState>>,
    mut transformst_next: ResMut<NextState<TransformType>>,
    q_sel: Query<Entity, With<Selected>>,
    q_valid_sel: Query<Entity, With<PickableElement>>,
    mut evtw_mergeload: EventWriter<MergeLoadEvent>,
    mut evtw_load: EventWriter<LoadEvent>,
) {
    let evt_keys = evt_keys.read().collect::<Vec<&KeyboardInput>>();
    let is_move = evt_keys.iter().any(|ki| ki.key_code == MOVE_KEY);
    let is_copy = evt_keys.iter().any(|ki| ki.key_code == COPY_KEY);
    if is_move || is_copy {
        // check if valid (something selected)
        if !q_sel.is_empty() {
            debug!("selecting transform tool");
            toolst_next.set(SchematicToolState::Transform);
            transformst_next.set(if is_move {
                debug!("selecting transform::move");
                TransformType::Move
            } else {
                debug!("selecting transform::copy");
                TransformType::Copy
            });
            evtw_mergeload.send(MergeLoadEvent);
            evtw_load.send(LoadEvent);
        }
    } else if evt_keys
        .iter()
        .any(|ki| ki.key_code == WIRE_TOOL_KEY && ki.state == ButtonState::Released)
    {
        toolst_next.set(SchematicToolState::Wiring);
    } else if evt_keys
        .iter()
        .any(|ki| ki.key_code == DEVICE_SPAWN_TOOL_KEY && ki.state == ButtonState::Released)
    {
        debug!("selecting device spawn tool");
        toolst_next.set(SchematicToolState::DeviceSpawn);
    }
    // } else if evt_keys
    //     .iter()
    //     .any(|ki| ki.key_code == KeyCode::KeyA && ki.state == ButtonState::Released)
    // {
    //     // select all
    //     let valids = q_valid_sel
    //         .iter()
    //         .map(|e| (e, Selected))
    //         .collect::<Vec<(Entity, Selected)>>();
    //     commands.insert_or_spawn_batch(valids.into_iter());
    // }
}

/// on mouse button released: add selected marker to all schematic elements with picked marker
fn main(
    mut e_newsc: EventReader<NewSnappedCursorPos>,
    qc: Query<&SchematicCursor>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut e_newpc: EventWriter<NewPickingCollider>,
    mut selres: ResMut<SelToolRes>,
    mut q_s: Query<&mut CompositeMeshData, With<SelMarker>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut e_sel: EventWriter<SelectEvt>,
    qcam: Query<&OrthographicProjection, (With<SchematicCamera>, Changed<OrthographicProjection>)>,
) {
    // record cursor left click location
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(coords) = &qc.single().coords {
            selres.sel_area_origin = coords.get_snapped_coords_float();
        };
    }
    let mut cmdata = q_s.single_mut();
    // send new picking collider event if cursor moves
    if let Some(NewSnappedCursorPos(Some(coords))) = e_newsc.read().last() {
        if buttons.pressed(MouseButton::Left) {
            // update selection area appearance
            new_valid_path(
                &mut cmdata,
                selres.sel_area_origin,
                coords.get_snapped_coords_float(),
            );
            e_newpc.send(NewPickingCollider::min_max(
                selres.sel_area_origin,
                coords.get_snapped_coords_float(),
            ));
        } else {
            e_newpc.send(NewPickingCollider::point(coords.get_snapped_coords_float()));
        }
    }
    if let Ok(p) = qcam.get_single() {
        new_stroke(&mut cmdata, p.scale);
    }

    if buttons.just_released(MouseButton::Left) {
        // if button is not held down: remove the selection visual
        remove_path(&mut cmdata);
        e_sel.send(SelectEvt::New);
    }

    if keys.just_released(KeyCode::Escape) {
        e_sel.send(SelectEvt::Clear);
    }

    if keys.just_released(KeyCode::KeyA) {
        e_sel.send(SelectEvt::All);
    }
}

/// updates the path in TessInData of the area selection entity to get visual
fn new_valid_path(cmdata: &mut CompositeMeshData, og_coords: Vec2, coords: Vec2) {
    let mut path_builder = bevyon::path_builder();
    path_builder.add_rectangle(
        &lyon_tessellation::geom::Box2D {
            min: lyon_tessellation::geom::Point::new(og_coords.x, og_coords.y),
            max: lyon_tessellation::geom::Point::new(coords.x, coords.y),
        },
        lyon_tessellation::path::Winding::Positive,
    );
    let path = Some(path_builder.build());

    for submesh in cmdata.iter_mut() {
        submesh.tess_data.path = path.clone();
    }
}
const WIDTH: f32 = 1.0;
/// updates the stroke options in TessInData of the area selection entity
fn new_stroke(cmdata: &mut CompositeMeshData, pscale: f32) {
    for submesh in cmdata.iter_mut() {
        submesh.tess_data.stroke = submesh.tess_data.stroke.map(|mut so| {
            so.line_width = WIDTH * pscale;
            so
        });
    }
}

/// udpates the path in TessInData of the area selection entity to be None (hidden)
fn remove_path(cmdata: &mut CompositeMeshData) {
    for submesh in cmdata.iter_mut() {
        submesh.tess_data.path = None;
    }
}

const Z_DEPTH: f32 = 0.8;
/// this function creates the selection rectangle mesh
fn setup(mut commands: Commands, mut materials: ResMut<Assets<SchematicMaterial>>) {
    let tess_fill_data = TessInData {
        path: None,
        stroke: None,
        fill: None,
        // fill: Some(bevyon::FillOptions::DEFAULT),  // TODO renable fill once transparency is working
    };
    let tess_stroke_data = TessInData {
        path: None,
        stroke: Some(bevyon::StrokeOptions::DEFAULT),
        fill: None,
    };
    commands.spawn(SelToolMarkerBundle {
        marker: SelMarker,
        tess_data: CompositeMeshData {
            zoom_invariant: false,
            mesh_data: vec![
                SubMesh::new_with_color(tess_fill_data, Color::WHITE.with_alpha(0.1)),
                SubMesh::new_with_color(
                    tess_stroke_data,
                    bevy::prelude::Color::Srgba(basic_colors::RED.with_alpha(1.0)),
                ),
            ],
        },
        mat_bundle: MaterialMesh2dBundle {
            material: materials.add(SchematicMaterial {
                color: Color::BLACK.with_alpha(0.0).into(),
            }),
            //            material: eres.mat_dflt.clone().unwrap(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, Z_DEPTH)),
            ..Default::default()
        },
    });
}
