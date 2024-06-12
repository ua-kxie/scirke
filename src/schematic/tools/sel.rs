use bevy::{math::bounding::Aabb2d, prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    bevyon::{self, CompositeMeshData, SubMesh, TessInData},
    schematic::{
        camera::SchematicCamera,
        elements::{SchematicElement, Selected},
        guides::{NewSnappedCursorPos, SchematicCursor},
        material::SchematicMaterial,
    },
};

use super::SchematicToolState;

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
    // Inverse,  // invert selected entities
    Clear, // deselect all
           // All,  // select all valid targets
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
            Update,
            (main, listener).run_if(in_state(SchematicToolState::Idle)),
        );
        app.init_resource::<SelToolRes>();
        app.add_event::<NewPickingCollider>();
        app.add_event::<SelectEvt>();
    }
}

/// this system listens to user inputs valid during idle
fn listener(
    keys: Res<ButtonInput<KeyCode>>,
    qc: Query<Entity, With<Selected>>,
    mut commands: Commands,
) {
    if keys.just_released(KeyCode::Delete) {
        for e in qc.iter() {
            commands.entity(e).despawn();
        }
    }
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
    if let Some(NewSnappedCursorPos(Some((_, float_coords)))) = e_newsc.read().last() {
        if buttons.pressed(MouseButton::Left) {
            // update selection area appearance
            new_valid_path(&mut cmdata, selres.sel_area_origin, *float_coords);
            e_newpc.send(NewPickingCollider::min_max(
                selres.sel_area_origin,
                *float_coords,
            ));
        } else {
            e_newpc.send(NewPickingCollider::point(*float_coords));
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
                SubMesh::new_with_color(tess_fill_data, Color::WHITE.with_a(0.1)),
                SubMesh::new_with_color(tess_stroke_data, Color::RED.with_a(1.0)),
            ],
        },
        mat_bundle: MaterialMesh2dBundle {
            material: materials.add(SchematicMaterial {
                color: Color::BLACK.with_a(0.0),
            }),
            //            material: eres.mat_dflt.clone().unwrap(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, Z_DEPTH)),
            ..Default::default()
        },
    });
}

pub fn select_all(q_valid: Query<Entity, With<SchematicElement>>, mut commands: Commands) {
    for e in q_valid.iter() {
        commands.entity(e).insert(Selected);
    }
}
