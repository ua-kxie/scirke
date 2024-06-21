use bevy::prelude::*;

use crate::schematic::electrical::{DefaultDevices, DeviceType};

use super::{transform::TransformType, SchematicToolState};

pub struct DeviceSpawnToolPlugin;

impl Plugin for DeviceSpawnToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            main.run_if(in_state(SchematicToolState::DeviceSpawn)),
        );
    }
}

fn main(
    keys: Res<ButtonInput<KeyCode>>,
    mut ntool_st: ResMut<NextState<SchematicToolState>>,
    mut ntransform_st: ResMut<NextState<TransformType>>,
    dd: Res<DefaultDevices>,
    mut e_newd: EventWriter<DeviceType>,
) {
    if keys.just_pressed(KeyCode::KeyG)
        || keys.just_pressed(KeyCode::KeyV)
        || keys.just_pressed(KeyCode::KeyI)
        || keys.just_pressed(KeyCode::KeyR)
        || keys.just_pressed(KeyCode::KeyL)
        || keys.just_pressed(KeyCode::KeyC)
    {
        // spawn device as child of cursor
        e_newd.send(if keys.just_pressed(KeyCode::KeyV) {
            dd.voltage_source()
        } else if keys.just_pressed(KeyCode::KeyI) {
            dd.current_source()
        } else if keys.just_pressed(KeyCode::KeyR) {
            dd.resistor()
        } else if keys.just_pressed(KeyCode::KeyL) {
            dd.inductor()
        } else if keys.just_pressed(KeyCode::KeyC) {
            dd.capacitor()
        } else {
            dd.gnd()
        });
        ntool_st.set(SchematicToolState::Transform);
        ntransform_st.set(TransformType::Copy);
    }
}
