/*
tool for drawing wires
*/

use bevy::prelude::*;

use crate::schematic::{
    guides::{NewSnappedCursor, SchematicCursor},
    material::WireMaterial,
};

use super::SchematicToolState;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum WireToolState {
    #[default]
    Ready, // tool is activated and ready to draw another wire
    Drawing(Entity), // tool is activated and in the process of drawing a wire segment
}

pub struct WireToolPlugin;

impl Plugin for WireToolPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, setup);
        app.init_state::<WireToolState>();
        app.add_systems(OnExit(SchematicToolState::Wiring), tool_cleanup);
        app.add_systems(Update, main.run_if(in_state(SchematicToolState::Wiring)));
    }
}
use crate::schematic::elements::create_lineseg;
fn main(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    wiretoolstate: Res<State<WireToolState>>,
    mut next_wiretoolstate: ResMut<NextState<WireToolState>>,
    mut next_schematictoolstate: ResMut<NextState<SchematicToolState>>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<WireMaterial>>,
    qc: Query<&SchematicCursor>,
    mut qt: Query<&mut Transform>,
    mut e_newsc: EventReader<NewSnappedCursor>,
) {
    // system should be set to run if schematic tool state is wiring
    // main purpose is to manage WireToolState
    let sc = qc.single();
    let Some(coords) = &sc.coords else { return };
    if keys.just_released(KeyCode::Escape) {
        next_schematictoolstate.set(SchematicToolState::Idle);
        return;
    }
    match wiretoolstate.get() {
        WireToolState::Ready => {
            if buttons.just_released(MouseButton::Left) {
                // create wire entity and set next wire tool state
                let ent = create_lineseg(commands, materials, meshes, coords.snapped_world_coords);
                next_wiretoolstate.set(WireToolState::Drawing(ent));
                return;
            }
        }
        WireToolState::Drawing(ent) => {
            if buttons.just_released(MouseButton::Left) {
                // persist ent
                next_wiretoolstate.set(WireToolState::Ready);
                return;
            }
            let Some(NewSnappedCursor(Some(coords))) = e_newsc.read().last() else {
                return;
            };
            let Ok(mut t) = qt.get_mut(*ent) else { return };
            *t = t.with_translation(coords.extend(0.0));
        }
    }
}

fn tool_cleanup(
    mut next_wiretoolstate: ResMut<NextState<WireToolState>>,
    wiretoolstate: ResMut<State<WireToolState>>,
    mut commands: Commands,
) {
    // this system is designed to be run everytime the tool is deactivated
    match wiretoolstate.get() {
        WireToolState::Ready => {}
        WireToolState::Drawing(ent) => {
            commands.entity(*ent).despawn();
            next_wiretoolstate.set(WireToolState::Ready);
        }
    }
}
