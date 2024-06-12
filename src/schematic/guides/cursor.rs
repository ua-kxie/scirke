//! this module takes cursor movement and updates the in-world cursor entity and send out cursor position changed events
//!

use bevy::{
    input::mouse::MouseMotion, math::vec3, prelude::*, sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
};
use lyon_tessellation::geom::euclid::{Box2D, Point2D};

use crate::{
    bevyon::{self, CompositeMeshData, TessInData},
    schematic::material::SchematicMaterial,
};

use super::SchematicCamera;

/// event indicating a new cursor position. None indicates that the cursor moved off-window
#[derive(Event, Deref, PartialEq)]
pub struct NewCursorPos(pub Option<Vec2>);

/// event indicating a new snapped cursor position. None indicates that the cursor moved off-window
/// contains the position in IVec2 and Vec2
#[derive(Event, Deref, PartialEq)]
pub struct NewSnappedCursorPos(pub Option<(IVec2, Vec2)>);

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        // cursor position and moved events need to be ready for other systems, so put in PreUpdate schedule
        app.add_systems(PreUpdate, update.run_if(on_event::<MouseMotion>()));
        app.add_systems(Update, redraw);
        app.add_event::<NewSnappedCursorPos>();
        app.add_event::<NewCursorPos>();
    }
}

/// component to store cursor position in various coordinates, if cursor is on window
/// a unique entity with this component represents the user's in-world cursor.
#[derive(Component)]
pub struct SchematicCursor {
    pub coords: Option<Coords>,
    snap_step: f32, // TODO should this be moved into a resource so snap step is sync'd?
}

impl Default for SchematicCursor {
    fn default() -> Self {
        Self {
            coords: None,
            snap_step: 1.0,
        }
    }
}

/// struct to collect coordinates in which to store cursor position
#[allow(dead_code)]
#[derive(Clone)]
pub struct Coords {
    vport_pos: Vec2,
    snapped_world_pos: (IVec2, Vec2),
    world_pos: Vec2,
}

#[allow(dead_code)]
impl Coords {
    pub fn get_coords(&self) -> Vec2 {
        self.world_pos
    }
    pub fn get_snapped_coords(&self) -> IVec2 {
        self.snapped_world_pos.0
    }
    pub fn get_snapped_coords_float(&self) -> Vec2 {
        self.snapped_world_pos.1
    }
}

/// bundle necessary for schematic cursor
#[derive(Bundle)]
struct CursorBundle {
    tess_data: CompositeMeshData,
    mat_bundle: MaterialMesh2dBundle<SchematicMaterial>,
    cursor: SchematicCursor,
}

/// z depth to alleviate z-fighting
const Z_DEPTH: f32 = 0.9;

/// this system updates the cursor entity and send out related events if applicable
fn update(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<SchematicCamera>>,
    mut q_cursor: Query<(&mut SchematicCursor, &mut Visibility, &mut Transform)>,
    mut e_new_snapped_curpos: EventWriter<NewSnappedCursorPos>,
    mut e_new_curpos: EventWriter<NewCursorPos>,
) {
    let (mut c, mut visibility, mut c_t) = q_cursor.single_mut();
    let (cam, cgt) = q_camera.get_single().unwrap();
    let window = q_window.get_single().unwrap();
    // let new_coords;
    let opt_coords = window
        .cursor_position()
        .map(|vport_pos| {
            cam.viewport_to_world_2d(cgt, vport_pos).map(|world_pos| {
                let snapped_world_pos = (
                    ((world_pos / c.snap_step).round() * c.snap_step).as_ivec2(),
                    (world_pos / c.snap_step).round() * c.snap_step,
                );
                Coords {
                    vport_pos,
                    snapped_world_pos,
                    world_pos,
                }
            })
        })
        .flatten();
    // send out event for new cursor world position
    e_new_curpos.send(NewCursorPos(
        opt_coords.clone().map(|coords| coords.world_pos),
    ));
    // send out event for new cursor snapped world position, if necessary
    if opt_coords.as_ref().map(|coords| coords.snapped_world_pos)
        != c.coords.as_ref().map(|coords| coords.snapped_world_pos)
    {
        let mut new_visibility = Visibility::Hidden;
        e_new_snapped_curpos.send(NewSnappedCursorPos(opt_coords.as_ref().map(|coords| {
            new_visibility = Visibility::Visible;
            *c_t = c_t.with_translation(coords.snapped_world_pos.1.extend(c_t.translation.z));
            (coords.snapped_world_pos.0, coords.snapped_world_pos.1)
        })));
        *visibility = new_visibility;
    };
    c.coords = opt_coords;
}

/// system to redraw cursor mesh as needed
fn redraw(
    q_projection: Query<&OrthographicProjection, Changed<OrthographicProjection>>,
    mut q_cursor: Query<&mut CompositeMeshData, With<SchematicCursor>>,
) {
    let Ok(projection) = q_projection.get_single() else {
        return;
    };
    let scale = projection.scale;
    let mut mesh_data = q_cursor.single_mut();
    *mesh_data = create_mesh_data(scale);
}

/// initialize [`SchematicCursor`] entity
fn setup(mut commands: Commands, mut materials: ResMut<Assets<SchematicMaterial>>) {
    let scale = 0.1; // default projection scale
    commands.spawn(CursorBundle {
        tess_data: create_mesh_data(scale),
        mat_bundle: MaterialMesh2dBundle {
            material: materials.add(SchematicMaterial {
                color: Color::BLACK.with_a(0.0),
            }),
            transform: Transform::from_translation(vec3(0.0, 0.0, Z_DEPTH)),
            ..Default::default()
        },
        cursor: SchematicCursor::default(),
    });
}

/// create mesh data for [`SchematicCursor`]
fn create_mesh_data(scale: f32) -> CompositeMeshData {
    let mut path_builder = bevyon::path_builder();
    let size = 4.0 * scale;
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
        stroke: Some(bevyon::StrokeOptions::DEFAULT.with_line_width(1.0 * scale)),
        fill: None,
    };
    CompositeMeshData::from_single(tessellator_input_data)
}
