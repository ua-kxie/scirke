/*
render into clip space to keep dimensions invariant of zoom
tessellate based on calculated clip position and size
cursor - canvas - clip
*/
use bevy::{math::vec3, prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};
use lyon_tessellation::geom::euclid::{Box2D, Point2D};

use crate::bevyon::{self, CompositeMeshData, SubMesh, TessInData};

use super::{SchematicCamera, ZoomInvariant};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, update);
    }
}

#[derive(Component)]
pub struct SchematicCursor {
    pub coords: Option<Coords>,
    snap_step: f32,
}

impl Default for SchematicCursor {
    fn default() -> Self {
        Self {
            coords: None,
            snap_step: 1.0,
        }
    }
}

pub struct Coords {
    pub screen_coords: Vec2,
    pub snapped_world_coords: Vec2,
    pub world_coords: Vec2,
    pub ndc_coords: Vec3,
}

#[derive(Bundle)]
struct CursorBundle {
    tess_data: CompositeMeshData,
    mat_bundle: MaterialMesh2dBundle<ColorMaterial>,
    cursor: SchematicCursor,
    zoom_invariant: ZoomInvariant,
}

const Z_DEPTH: f32 = 1.0;

/// mixing in of snapping here isn't ideal - leave for now
fn update(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<SchematicCamera>>,
    mut q_cursor: Query<(&mut SchematicCursor, &mut Visibility, &mut Transform)>,
) {
    let (mut c, mut visibility, mut c_t) = q_cursor.single_mut();
    let cam = q_camera.get_single();
    let window = q_window.get_single();
    if cam.is_ok() && window.as_ref().is_ok_and(|w| w.cursor_position().is_some()) {
        let (cam, cgt) = cam.unwrap();
        let window = window.unwrap();
        let screen_coords = window.cursor_position().unwrap();
        if let Some(world_coords) = cam.viewport_to_world_2d(cgt, screen_coords) {
            let ndc_coords = cam
                .world_to_ndc(cgt, world_coords.extend(c_t.translation.z))
                .unwrap();
            let snapped_world_coords = (world_coords / c.snap_step).round() * c.snap_step;
            *visibility = Visibility::Visible;
            c.coords = Some(Coords {
                screen_coords,
                world_coords,
                ndc_coords,
                snapped_world_coords,
            });

            *c_t = c_t.with_translation(snapped_world_coords.extend(c_t.translation.z));
        } else {
            *visibility = Visibility::Hidden;
            c.coords = None;
        }
    } else {
        *visibility = Visibility::Hidden;
        c.coords = None;
    }
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let mut path_builder = bevyon::path_builder();
    let size = 4.0;
    path_builder.add_rectangle(
        &Box2D {
            min: Point2D::splat(-size),
            max: Point2D::splat(size),
        },
        lyon_tessellation::path::Winding::Positive,
    );
    let path = Some(path_builder.build());

    let tessellator_input_data = TessInData {
        path,
        stroke: Some(
            bevyon::StrokeOptions::DEFAULT
                .with_line_width(2.0)
                .with_tolerance(1.0),
        ),

        fill: None,
    };
    commands.spawn(CursorBundle {
        tess_data: CompositeMeshData::from_single(tessellator_input_data),
        mat_bundle: MaterialMesh2dBundle {
            material: materials.add(Color::GREEN),
            transform: Transform::from_translation(vec3(0.0, 0.0, Z_DEPTH)),
            ..Default::default()
        },
        cursor: SchematicCursor::default(),
        zoom_invariant: ZoomInvariant,
    });
}
