use bevy::{math::bounding::Aabb2d, prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    bevyon::{self, CompositeMeshData, SubMesh, TessInData},
    schematic::{
        guides::{NewSnappedCursor, SchematicCursor},
        material::SchematicMaterial,
    },
};

/// struct describes the picking collider
pub enum PickingCollider {
    Point(Vec2),
    AreaIntersect(Aabb2d),
    AreaContains(Aabb2d),
}

/// event to be sent when the picking collider changes
#[derive(Event)]
pub struct NewPickingCollider(PickingCollider);

impl NewPickingCollider {
    pub fn point(p: Vec2) -> NewPickingCollider {
        NewPickingCollider(PickingCollider::Point(p))
    }
    pub fn min_max(min: Vec2, max: Vec2) -> NewPickingCollider {
        let a = max - min;
        match (a.x * a.y).is_sign_positive() {
            true => NewPickingCollider(PickingCollider::AreaContains(Aabb2d { min, max })),
            false => NewPickingCollider(PickingCollider::AreaIntersect(Aabb2d { min, max })),
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

#[derive(Resource, Default)]
struct SelToolRes {
    sel_area_origin: Vec2,
}

// system should work with SchematicElement, GlobalTransform, to determine if colliding with picking collider
// picking system: get all schematicElements, cursor snapped position, and mark picked elements as such
// based on event: pickingcollider changed
fn pick(mut e_new_collider: EventReader<NewPickingCollider>) {
    let Some(NewPickingCollider(collider)) = e_new_collider.read().last() else {
        return;
    };
    match collider {
        PickingCollider::Point(_) => todo!(),
        PickingCollider::AreaIntersect(_) => todo!(),
        PickingCollider::AreaContains(_) => todo!(),
    }
}

pub struct SelToolPlugin;

impl Plugin for SelToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, main);
        app.init_resource::<SelToolRes>();
        app.add_event::<NewPickingCollider>();
    }
}

fn main(
    mut e_newsc: EventReader<NewSnappedCursor>,
    qc: Query<&SchematicCursor>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut e_newpc: EventWriter<NewPickingCollider>,
    mut selres: ResMut<SelToolRes>,
    mut q_s: Query<&mut CompositeMeshData, With<SelMarker>>,
) {
    // record cursor left click location
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(coords) = &qc.single().coords {
            selres.sel_area_origin = coords.snapped_world_coords;
        };
    }
    let mut cmdata = q_s.single_mut();
    // send new picking collider event if cursor moves
    if let Some(NewSnappedCursor(Some(coords))) = e_newsc.read().last() {
        if buttons.pressed(MouseButton::Left) {
            // update selection area appearance
            new_valid_path(&mut cmdata, selres.sel_area_origin, *coords);
            e_newpc.send(NewPickingCollider::min_max(selres.sel_area_origin, *coords));
        } else {
            e_newpc.send(NewPickingCollider::point(*coords));
        }
    }

    if !buttons.pressed(MouseButton::Left) {
        remove_path(&mut cmdata);
    }
}

/// udpates the path in TessInData of the area selection entity to get visual
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
        fill: Some(bevyon::FillOptions::DEFAULT),
    };
    // TODO: stroke width needs to scale with projection scale so it appears zoom invariant
    let tess_stroke_data = TessInData {
        path: None,
        stroke: Some(
            bevyon::StrokeOptions::DEFAULT
        ),
        fill: None,
    };
    commands.spawn(SelToolMarkerBundle {
        marker: SelMarker,
        tess_data: CompositeMeshData {
            mesh_data: vec![
                SubMesh::new_with_color(tess_fill_data, Color::WHITE.with_a(0.1)),
                SubMesh::new_with_color(tess_stroke_data, Color::RED.with_a(1.0)),
            ],
        },
        mat_bundle: MaterialMesh2dBundle {
            material: materials.add(SchematicMaterial {
                color: Color::BLACK.with_a(0.0),
            }),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, Z_DEPTH)),
            ..Default::default()
        },
    });
}
