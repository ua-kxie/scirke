/*
render into clip space to keep dimensions invariant of zoom
tessellate based on calculated clip position and size
cursor - canvas - clip
*/
use bevy::{math::vec3, prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};
use lyon_tessellation::geom::euclid::{Box2D, Point2D};

use crate::{
    bevyon::{self, CompositeMeshData, TessInData},
    schematic::material::SchematicMaterial,
};

use super::{SchematicCamera, ZoomInvariant};

/// event indicating a new cursor position. None indicates that the cursor moved off-window
#[derive(Event, Deref, PartialEq)]
pub struct NewSnappedCursor(pub Option<Vec2>);

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, update.run_if(cursor_moved));
        app.add_event::<NewSnappedCursor>();
    }
}

fn cursor_moved(mut ecm: EventReader<CursorMoved>) -> bool {
    ecm.read().last().is_some()
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

impl SchematicCursor {
    fn as_new_event(&self) -> NewSnappedCursor {
        NewSnappedCursor(self.coords.as_ref().map(|x| x.snapped_world_coords))
    }
}

#[derive(Clone)]
pub struct Coords {
    screen_coords: Vec2,
    pub snapped_world_coords: Vec2,
    world_coords: Vec2,
    ndc_coords: Vec3,
}

#[derive(Bundle)]
struct CursorBundle {
    tess_data: CompositeMeshData,
    mat_bundle: MaterialMesh2dBundle<SchematicMaterial>,
    cursor: SchematicCursor,
    zoom_invariant: ZoomInvariant,
}

const Z_DEPTH: f32 = 1.0;

/// mixing in of snapping here isn't ideal - leave for now
fn update(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<SchematicCamera>>,
    mut q_cursor: Query<(&mut SchematicCursor, &mut Visibility, &mut Transform)>,
    mut e_new_snapped: EventWriter<NewSnappedCursor>,
) {
    let (mut c, mut visibility, mut c_t) = q_cursor.single_mut();
    let cam = q_camera.get_single();
    let window = q_window.get_single();
    let new_coords;
    let new_event;
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
            new_coords = Some(Coords {
                screen_coords,
                world_coords,
                ndc_coords,
                snapped_world_coords,
            });
            new_event = NewSnappedCursor(Some(snapped_world_coords));

            // snap the cursor position
            *c_t = c_t.with_translation(snapped_world_coords.extend(c_t.translation.z));
        } else {
            *visibility = Visibility::Hidden;
            new_coords = None;
            new_event = NewSnappedCursor(None);
        }
    } else {
        *visibility = Visibility::Hidden;
        new_coords = None;
        new_event = NewSnappedCursor(None);
    }
    // see whats changed and maybe write an event
    if c.as_new_event() != new_event {
        e_new_snapped.send(new_event);
    }
    // write the new coords
    c.coords = new_coords;
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<SchematicMaterial>>) {
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
            material: materials.add(SchematicMaterial {
                color: Color::BLACK.with_a(0.0),
            }),
            transform: Transform::from_translation(vec3(0.0, 0.0, Z_DEPTH)),
            ..Default::default()
        },
        cursor: SchematicCursor::default(),
        zoom_invariant: ZoomInvariant,
    });
}
