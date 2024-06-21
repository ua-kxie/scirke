use bevy::prelude::*;
use bevy_egui::{
    egui::{self, TextEdit},
    EguiContexts,
};

use crate::schematic::electrical::{DeviceParams, Selected};

pub fn params_ui(
    mut egui_context: EguiContexts,
    mut qs: Query<&mut DeviceParams, With<Selected>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
) {
    let ctx = egui_context.ctx_mut();
    egui::Window::new("params editor").show(ctx, |ui| {
        let mut temp = qs.get_single_mut();
        let Ok(param) = temp.as_deref_mut() else {
            return;
        };
        match param {
            DeviceParams::Raw(ref mut s) => {
                ui.add(
                    TextEdit::singleline(s)
                        .desired_width(f32::INFINITY)
                        .lock_focus(true)
                        .font(egui::TextStyle::Monospace),
                );
            }
            DeviceParams::Float(_f) => todo!(), // not supported yet
        }
    });
    if ctx.wants_keyboard_input() {
        keys.reset_all();
    }
    if ctx.wants_pointer_input() {
        mouse.reset_all();
    }
}
