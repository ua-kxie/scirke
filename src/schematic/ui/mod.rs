use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseWheel;
use bevy::{input::mouse::MouseButtonInput, prelude::*};

pub mod console;
mod params_editor;

#[derive(Resource, Default, Deref, DerefMut)]
struct UiHasFocus(bool);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
/// The UiSet for ui related systems
pub enum UiSet {
    /// Systems operating any ui
    Ui,

    /// after ui finished, e.g. system to consume input events if ui has focus
    PostUi,
}

pub struct SchematicUiPlugin;

impl Plugin for SchematicUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(console::ConsolePlugin);
        app.add_systems(Update, (
            params_editor::params_ui.in_set(UiSet::Ui),
            consume_input_events.in_set(UiSet::PostUi),
        ));
        app.init_resource::<UiHasFocus>();
        app.configure_sets(
            Update,
            (
                UiSet::Ui,
                UiSet::PostUi.after(UiSet::Ui),
            ),
        );
    }
}

/// consume set of input events if UI has focus, PostUI.
fn consume_input_events(
    res_has_focus: Res<UiHasFocus>,
    mut ev_mb: ResMut<Events<MouseButtonInput>>,
    mut ev_mw: ResMut<Events<MouseWheel>>,
    mut ev_ki: ResMut<Events<KeyboardInput>>,
    mut in_key: ResMut<ButtonInput<KeyCode>>,
    mut in_mb: ResMut<ButtonInput<MouseButton>>,
    /*
    https://bevy-cheatbook.github.io/builtins.html#input-events
    MouseButtonInput: Changes in the state of mouse buttons
    MouseWheel: Scrolling by a number of pixels or lines (MouseScrollUnit)
    MouseMotion: Relative movement of the mouse (pixels from previous frame), regardless of the OS pointer/cursor
    CursorMoved: New position of the OS mouse pointer/cursor
    KeyboardInput: Changes in the state of keyboard keys (keypresses, not text)
    ReceivedCharacter: Unicode text input from the OS (correct handling of the user's language and layout)
    Ime: Unicode text input from IME (support for advanced text input in different scripts)
    TouchInput: Change in the state of a finger touching the touchscreen
    GamepadEvent: Changes in the state of a gamepad or any of its buttons or axes
    GamepadRumbleRequest: Send these events to control gamepad rumble
    TouchpadMagnify: Pinch-to-zoom gesture on laptop touchpad (macOS)
    TouchpadRotate: Two-finger rotate gesture on laptop touchpad (macOS)
    */
) {
    if **res_has_focus {
        ev_mb.clear();
        ev_mw.clear();
        ev_ki.clear();
        in_key.reset_all();
        in_mb.reset_all();
    }
}

