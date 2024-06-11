#![doc = include_str ! ("../README.md")]
#![deny(missing_docs)]

use bevy::{input::keyboard::KeyboardInput, prelude::*};
use bevy_egui::EguiPlugin;

use crate::console::{console_ui, receive_console_line, ConsoleState};
pub use crate::console::{
    ConsoleCommandEntered, ConsoleConfiguration, ConsoleOpen, PrintConsoleLine,
};
pub use bevy_egui::egui::epaint::Color32;

mod console;

/// Console plugin.
pub struct ConsolePlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
/// The SystemSet for console/command related systems
pub enum ConsoleSet {
    /// Systems operating the console UI (the input layer)
    ConsoleUI,

    /// Systems executing console commands (the functionality layer).
    /// All command handler systems are added to this set
    Commands,

    /// Systems running after command systems, which depend on the fact commands have executed beforehand (the output layer).
    /// For example a system which makes use of [`PrintConsoleLine`] events should be placed in this set to be able to receive
    /// New lines to print in the same frame
    PostCommands,
}

/// Run condition which does not run any command systems if no command was entered
fn have_commands(commands: EventReader<ConsoleCommandEntered>) -> bool {
    !commands.is_empty()
}

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConsoleConfiguration>()
            .init_resource::<ConsoleState>()
            .init_resource::<ConsoleOpen>()
            .add_event::<PrintConsoleLine>()
            .add_event::<ConsoleCommandEntered>()
            .add_systems(
                Update,
                (
                    console_ui.in_set(ConsoleSet::ConsoleUI),
                    receive_console_line.in_set(ConsoleSet::PostCommands),
                ),
            )
            .configure_sets(
                Update,
                (
                    ConsoleSet::Commands
                        .after(ConsoleSet::ConsoleUI)
                        .run_if(have_commands),
                    ConsoleSet::PostCommands.after(ConsoleSet::Commands),
                ),
            );
        app.add_systems(
            Update,
            block_inputs_on_console_focus.in_set(ConsoleSet::PostCommands),
        );
        // Don't initialize an egui plugin if one already exists.
        // This can happen if another plugin is using egui and was installed before us.
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
    }
}

/// system to block inputs on console focus
pub fn block_inputs_on_console_focus(
    mut events: ResMut<Events<KeyboardInput>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    console_open: Res<ConsoleOpen>,
) {
    if console_open.open {
        events.drain().for_each(drop);
        keys.reset_all();
        mouse.reset_all();
    }
}
