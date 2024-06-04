use bevy::prelude::*;

mod sel;
mod transform;
mod wire;

use super::guides::SchematicCursor;
pub use sel::{NewPickingCollider, PickingCollider, SelectEvt};

const WIRE_TOOL_KEY: KeyCode = KeyCode::KeyW;
const MOVE_KEY: KeyCode = KeyCode::KeyM;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchematicToolState {
    #[default]
    Idle, // also select
    Wiring, // for drawing wires
    Transform(IVec2), // moving elements around, IVec2 stores snapped cursor position when entering state
                      // Label,   // wire/net labeling
                      // Comment, // plain text comment with basic formatting options
}

pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            wire::WireToolPlugin,
            sel::SelToolPlugin,
            transform::TransformToolPlugin,
        ));
        app.add_systems(Update, main);
        app.init_state::<SchematicToolState>();
    }
}

fn main(
    keys: Res<ButtonInput<KeyCode>>,
    curr_toolstate: Res<State<SchematicToolState>>,
    mut next_toolstate: ResMut<NextState<SchematicToolState>>,
    c: Query<&SchematicCursor>,
) {
    if keys.just_released(KeyCode::Escape) {
        next_toolstate.set(SchematicToolState::Idle);
        return;
    }
    match curr_toolstate.get() {
        SchematicToolState::Idle => {
            if keys.just_released(MOVE_KEY) {
                if let Some(coords) = &c.single().coords {
                    next_toolstate.set(SchematicToolState::Transform(
                        coords.snapped_world_coords.as_ivec2(),
                    ));
                }
            } else if keys.just_released(WIRE_TOOL_KEY) {
                next_toolstate.set(SchematicToolState::Wiring);
            }
        }
        SchematicToolState::Wiring => {}
        SchematicToolState::Transform(_) => {}
    }
}
