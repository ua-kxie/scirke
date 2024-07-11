//! this module takes cursor movement and updates the in-world cursor entity and send out cursor position changed events
//! the new cursor position is calculated and event sent in preupdate, but the cursor entity is updated in postupdate
//! this enables systems to choose whether to operate with the rendered cursor position (in cursor entity) or the new data
//! sent out in events

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
pub struct NewCursorPos(pub SchCurPos);

/// event indicating a new snapped cursor position. None indicates that the cursor moved off-window
/// contains the position in IVec2 and Vec2
#[derive(Event, Deref, PartialEq)]
pub struct NewSnappedCursorPos(pub SchCurPos);

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        // cursor position and moved events need to be ready for other systems, so put in PreUpdate schedule
        app.add_systems(PreUpdate, send_events.run_if(on_event::<MouseMotion>()));
        app.add_systems(Update, redraw);
        app.add_systems(PostUpdate, post_update);
        app.add_event::<NewSnappedCursorPos>();
        app.add_event::<NewCursorPos>();
    }
}

/// component to store cursor position in various coordinates, if cursor is on window
/// a unique entity with this component represents the user's in-world cursor.
#[derive(Component)]
pub struct SchematicCursor {
    pub coords: SchCurPos,
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

/// schematic cursor position
type SchCurPos = Option<Coords>;

/// struct to collect coordinates in which to store cursor position
#[allow(dead_code)]
#[derive(Clone, PartialEq)]
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
fn send_events(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<SchematicCamera>>,
    q_cursor: Query<&SchematicCursor>,
    mut e_new_snapped_curpos: EventWriter<NewSnappedCursorPos>,
    mut e_new_curpos: EventWriter<NewCursorPos>,
) {
    let c = q_cursor.single();
    let (cam, cgt) = q_camera.get_single().unwrap();
    let window = q_window.get_single().unwrap();
    // let new_coords;
    let sch_cur_pos = window
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
    // send out event for new cursor snapped world position, if necessary
    if sch_cur_pos.as_ref().map(|coords| coords.snapped_world_pos)
        != c.coords.as_ref().map(|coords| coords.snapped_world_pos)
    {
        e_new_snapped_curpos.send(NewSnappedCursorPos(sch_cur_pos.clone()));
        // let mut new_visibility = Visibility::Hidden;
        // e_new_snapped_curpos.send(NewSnappedCursorPos(sch_cur_pos.as_ref().map(|coords| {
        //     // new_visibility = Visibility::Visible;
        //     // *c_t = c_t.with_translation(coords.snapped_world_pos.1.extend(c_t.translation.z));
        //     (coords.snapped_world_pos.0, coords.snapped_world_pos.1)
        // })));
        // *visibility = new_visibility;
    };
    // send out event for new cursor world position
    e_new_curpos.send(NewCursorPos(sch_cur_pos));
    // c.coords = opt_coords;
}

/// updates schematic cursor entity
fn post_update(
    mut q_cursor: Query<(&mut SchematicCursor, &mut Transform, &mut Visibility)>,
    mut e_new_curpos: EventReader<NewCursorPos>,
    mut e_new_snapped_curpos: EventReader<NewSnappedCursorPos>,
) {
    if let Some(NewCursorPos(sch)) = e_new_curpos.read().last() {
        let (mut c, mut t, mut v) = q_cursor.single_mut();
        if let Some(NewSnappedCursorPos(sch)) = e_new_snapped_curpos.read().last() {
            let mut new_visibility = Visibility::Hidden;
            sch.as_ref().map(|coords| {
                *t = t.with_translation(coords.snapped_world_pos.1.extend(t.translation.z));
                new_visibility = Visibility::Visible;
            });
            *v = new_visibility;
        }
        c.coords = sch.clone();
    }
}

/// system to redraw cursor mesh when zoom changes, such that it appears zoom invariant
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
                color: Color::BLACK.with_alpha(0.0).into(),
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
