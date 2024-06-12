/*
tool for drawing wires
*/

use bevy::prelude::*;

use crate::schematic::{
    elements::{self, ElementsRes, Preview, SchematicElement},
    guides::{NewSnappedCursorPos, SchematicCursor},
};

use super::SchematicToolState;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum WireToolState {
    #[default]
    Ready, // tool is activated and ready to draw another wire
    Drawing(IVec2), // tool is activated and in the process of drawing a wire segment
}

pub struct WireToolPlugin;

impl Plugin for WireToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<WireToolState>();
        app.add_systems(Update, main.run_if(in_state(SchematicToolState::Wiring)));
        app.add_systems(OnExit(SchematicToolState::Wiring), cleanup);
    }
}

/// resets the tool so that it is ready for next initiation
fn cleanup(mut nwts: ResMut<NextState<WireToolState>>) {
    nwts.set(WireToolState::Ready);
}

fn main(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    wiretoolstate: Res<State<WireToolState>>,
    mut next_wiretoolstate: ResMut<NextState<WireToolState>>,
    mut next_schematictoolstate: ResMut<NextState<SchematicToolState>>,
    mut commands: Commands,
    qc: Query<&SchematicCursor>,
    mut e_newsc: EventReader<NewSnappedCursorPos>,
    eres: Res<ElementsRes>,
    eqsp: Query<Entity, (With<SchematicElement>, With<Preview>)>,
    mut prev_curpos: Local<IVec2>,
) {
    // system should be set to run if schematic tool state is wiring
    // main purpose is to manage WireToolState
    let sc = qc.single();
    let Some(coords) = &sc.coords else { return };
    if keys.just_released(KeyCode::Escape) {
        elements::despawn_preview(&mut commands, &eqsp);
        next_schematictoolstate.set(SchematicToolState::Idle);
        return;
    }
    let now_curpos = coords.get_snapped_coords();
    match wiretoolstate.get() {
        WireToolState::Ready => {
            if buttons.just_pressed(MouseButton::Left) {
                next_wiretoolstate.set(WireToolState::Drawing(*prev_curpos));
            }
        }
        WireToolState::Drawing(src) => {
            if buttons.just_pressed(MouseButton::Left) {
                elements::persist_preview(&mut commands, &eqsp);
                next_wiretoolstate.set(WireToolState::Drawing(*prev_curpos));
            } else if let Some(NewSnappedCursorPos(Some(_))) = e_newsc.read().last() {
                elements::despawn_preview(&mut commands, &eqsp);
                compute_preview(&mut commands, eres, *src, now_curpos);
            }
        }
    }
    *prev_curpos = now_curpos;
}

/// this function pathfinds the optimal route from src to dst with a traversal cost function
/// TODO: add pathfinding; right now this function just forces wires to horz/vert.
fn route(
    src: IVec2,
    dst: IVec2,
    _traversal_cost: Option<()>, // function to return cost of routing over a particular coordinate, using Option as placeholder for now
) -> Vec<IVec2> {
    // basic routing: just go along x-axis, then y
    vec![src, IVec2::new(src.x, dst.y), dst]
}

/// this system computes the preview entities and adds them to world with preview
fn compute_preview(mut commands: &mut Commands, eres: Res<ElementsRes>, src: IVec2, dst: IVec2) {
    let path = route(src, dst, None);

    // filter redundant nodes - necessary to avoid solder dots where crossing another net segment
    let mut simple_path = Vec::with_capacity(path.len());
    simple_path.push(*path.first().unwrap());
    for i in 1..path.len() - 1 {
        // simply removes nodes bisecting a straigt line
        if (path[i - 1].x != path[i + 1].x) && (path[i - 1].y != path[i + 1].y) {
            simple_path.push(path[i]);
        }
    }
    simple_path.push(*path.last().unwrap());

    for i in 1..simple_path.len() {
        elements::create_preview_lineseg(
            &mut commands,
            &eres,
            simple_path[i - 1].as_vec2().extend(0.0),
            simple_path[i].as_vec2().extend(0.0),
        )
    }
}
