use bevy::math::vec2;
/*
render into clip space to keep dimensions invariant of zoom
tessellate based on calculated clip position and size
cursor - canvas - clip
*/
use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};
use lyon_tessellation::geom::euclid::{Box2D, Point2D};

use crate::bevyon::{self, TessInData};

use super::{camera::SchematicCamera, ClipMaterial, Snap};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, update);
    }
}

#[derive(Component, Default)]
pub struct SchematicCursor {
    pub coords: Option<Coords>,
}

pub struct Coords {
    pub screen_coords: Vec2,
    pub snapped_world_coords: Vec2,
    pub world_coords: Vec2,
    pub ndc_coords: Vec3,
}

#[derive(Bundle)]
struct CursorBundle {
    tess_data: TessInData,
    mat_bundle: MaterialMesh2dBundle<super::ClipMaterial>,
    snap: Snap,
    cursor: SchematicCursor,
}

const Z_DEPTH: f32 = 0.0;

/// mixing in of snapping here isn't ideal - leave for now
fn update(
    q_s: Query<&Snap, With<SchematicCursor>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<SchematicCamera>>,
    mut q_cursor: Query<(&mut SchematicCursor, &mut Visibility, &mut Transform)>,
) {
    let (mut c, mut visibility, mut c_t) = q_cursor.single_mut();
    let s = q_s.single();
    let cam = q_camera.get_single();
    let window = q_window.get_single();
    if cam.is_ok() && window.as_ref().is_ok_and(|w| w.cursor_position().is_some()) {
        let (cam, cgt) = cam.unwrap();
        let window = window.unwrap();
        let screen_coords = window.cursor_position().unwrap();
        if let Some(world_coords) = cam.viewport_to_world_2d(cgt, screen_coords) {
            let ndc_coords = cam.world_to_ndc(cgt, world_coords.extend(c_t.translation.z)).unwrap();

            *visibility = Visibility::Visible;
            c.coords = Some(Coords {
                screen_coords,
                world_coords,
                ndc_coords,
                snapped_world_coords: (world_coords / s.world_step).round() * s.world_step,
            });

            *c_t = c_t.with_translation(ndc_coords);
        } else {
            *visibility = Visibility::Hidden;
            c.coords = None;
        }
    } else {
        *visibility = Visibility::Hidden;
        c.coords = None;
    }
}

fn setup(mut commands: Commands, mut clip_materials: ResMut<Assets<ClipMaterial>>) {
    let mut path_builder = bevyon::path_builder();
    path_builder.add_rectangle(
        &Box2D {
            min: Point2D::splat(-0.01),
            max: Point2D::splat(0.01),
        },
        lyon_tessellation::path::Winding::Positive,
    );
    path_builder.add_rectangle(
        &Box2D {
            min: Point2D::splat(-0.02),
            max: Point2D::splat(0.02),
        },
        lyon_tessellation::path::Winding::Negative,
    );
    let path = Some(path_builder.build());

    let tessellator_input_data = TessInData {
        path,
        stroke: None,
        // stroke: Some(bevyon::StrokeOptions::DEFAULT.with_line_width(0.01).with_tolerance(0.001)),
        fill: Some(bevyon::FillOptions::DEFAULT),
        z_depth: Z_DEPTH,
    };
    commands.spawn(CursorBundle {
        tess_data: tessellator_input_data,
        mat_bundle: MaterialMesh2dBundle {
            material: clip_materials.add(ClipMaterial {
                z_depth: Z_DEPTH,
                color: Color::YELLOW,
            }),
            ..Default::default()
        },
        snap: Snap::DEFAULT_CLIP,
        cursor: SchematicCursor::default(),
    });
}
