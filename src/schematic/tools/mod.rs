use bevy::prelude::*;

mod wire;

const WIRE_TOOL_KEY: KeyCode = KeyCode::KeyW;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchematicToolState {
    #[default]
    Idle,  // also select
    Wiring,  // for drawing wires
    // Transform,  // moving elements around
    // Label,   // wire/net labeling
    // Comment, // plain text comment with basic formatting options
}

pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            wire::WireToolPlugin,
            // wire::SelToolPlugin,
        ));
        app.add_systems(Update, main);
        // app.add_systems(OnEnter(SchematicToolState::Idle), reset);

        app.init_state::<SchematicToolState>();
    }
}

fn main(
    keys: Res<ButtonInput<KeyCode>>,
    curr_toolstate: Res<State<SchematicToolState>>,
    mut next_toolstate: ResMut<NextState<SchematicToolState>>,
    // q_selected: Query<&WireSeg, With<Selected>>,
    mut commands: Commands,
    // mut e_clonetoent: EventWriter<CloneToEnt>,
    // q_cursor: Query<Entity, With<CursorMarker>>,
) {
    if keys.just_released(KeyCode::Escape) {
        next_toolstate.set(SchematicToolState::Idle);
        return;
    }
    match curr_toolstate.get() {
        SchematicToolState::Idle => {
            if keys.just_released(WIRE_TOOL_KEY) {
                next_toolstate.set(SchematicToolState::Wiring);
            }
        }
        SchematicToolState::Wiring => {}
    }
}