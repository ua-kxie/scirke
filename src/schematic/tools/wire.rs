/*
tool for drawing wires
*/

use bevy::prelude::*;

use crate::schematic::{
    elements::{self, ElementsRes, Preview, SchematicElement},
    guides::{NewSnappedCursor, SchematicCursor},
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
        // app.add_systems(Startup, setup);
        app.init_state::<WireToolState>();
        // app.add_systems(OnExit(SchematicToolState::Wiring), tool_cleanup);
        app.add_systems(Update, main.run_if(in_state(SchematicToolState::Wiring)));
    }
}
fn main(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    wiretoolstate: Res<State<WireToolState>>,
    mut next_wiretoolstate: ResMut<NextState<WireToolState>>,
    mut next_schematictoolstate: ResMut<NextState<SchematicToolState>>,
    mut commands: Commands,
    qc: Query<&SchematicCursor>,
    qt: Query<&mut Transform>,
    mut e_newsc: EventReader<NewSnappedCursor>,
    eres: Res<ElementsRes>,
    eqsp: Query<Entity, (With<SchematicElement>, With<Preview>)>,
) {
    // system should be set to run if schematic tool state is wiring
    // main purpose is to manage WireToolState
    let sc = qc.single();
    let Some(coords) = &sc.coords else { return };
    if keys.just_released(KeyCode::Escape) {
        elements::despawn_preview(&mut commands, eqsp);
        next_schematictoolstate.set(SchematicToolState::Idle);
        return;
    }
    match wiretoolstate.get() {
        WireToolState::Ready => {
            if buttons.just_released(MouseButton::Left) {
                next_wiretoolstate.set(WireToolState::Drawing(
                    coords.snapped_world_coords.as_ivec2(),
                ));
                return;
            }
        }
        WireToolState::Drawing(src) => {
            if buttons.just_released(MouseButton::Left) {
                // todo: persist preview elements
                elements::persist_preview(&mut commands, eqsp);
                next_wiretoolstate.set(WireToolState::Ready);
                return;
            }
            let Some(NewSnappedCursor(Some(dst))) = e_newsc.read().last() else {
                return;
            };
            elements::despawn_preview(&mut commands, eqsp);
            compute_preview(commands, eres, *src, dst.as_ivec2());
        }
    }
}

// fn tool_cleanup(
//     mut next_wiretoolstate: ResMut<NextState<WireToolState>>,
//     wiretoolstate: ResMut<State<WireToolState>>,
//     mut commands: Commands,
// ) {
//     // this system is designed to be run everytime the tool is deactivated
//     match wiretoolstate.get() {
//         WireToolState::Ready => {}
//         WireToolState::Drawing(ent) => {
//             commands.entity(*ent).despawn();
//             next_wiretoolstate.set(WireToolState::Ready);
//         }
//     }
// }

// wire routing: basic routing make wire go along x and y axis aligned
// variable entity count
// make marker component? preview marker component

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
fn compute_preview(mut commands: Commands, eres: Res<ElementsRes>, src: IVec2, dst: IVec2) {
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

// schematicToolSt
// WireToolSt
// on rdy: nothing
// during draw: just save src coordinate, call route
// delete preview, add new segments as preview
// on exit draw state: run pruning/assign id
