use bevy::{prelude::*, reflect::Enum};

use super::{
    camera::SchematicCamera, elements::SpId, guides::SchematicCursor, tools::SchematicToolState,
    SnapSet,
};

#[derive(Resource, Default)]
pub struct InfoRes {
    cpos: Option<IVec2>,
    scale: f32,
    toolst: SchematicToolState,
    picked: Option<Entity>,
}

impl InfoRes {
    fn line(&self) -> String {
        [
            format!("scale: {:.2e}; ", self.scale),
            if let Some(cpos) = self.cpos {
                format!("x: {:+03}; y: {:+03}; ", cpos.x, cpos.y)
            } else {
                format!("x: _; y: _; ")
            },
            format!("tool state: {}; ", self.toolst.variant_name()),
        ]
        .concat()
    }
    // pub fn set_scale(&mut self, scale: f32) {
    //     self.scale = scale;
    // }
    // pub fn set_cpos(&mut self, cpos: Option<IVec2>) {
    //     self.cpos = cpos;
    // }
    // pub fn set_toolst(&mut self, ts: SchematicToolState) {
    //     self.toolst = ts;
    // }
    pub fn set_picked(&mut self, e: Option<Entity>) {
        self.picked = e
    }
}

#[derive(Component)]
struct InfoTextMarker;

pub struct InfoPlugin;

impl Plugin for InfoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InfoRes>();
        app.add_systems(Startup, setup);
        app.add_systems(PostUpdate, update.after(SnapSet));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
        InfoTextMarker,
    ));
}

fn update(
    mut text: Query<&mut Text, With<InfoTextMarker>>,
    cursor: Query<&SchematicCursor>,
    projection: Query<&OrthographicProjection, With<SchematicCamera>>,
    toolst: Res<State<SchematicToolState>>,
    mut infores: ResMut<InfoRes>,
    // devices: Res<Assets<DeviceType>>,
    // qd: Query<&Device>,
    qid: Query<&SpId>,
) {
    infores.cpos = cursor
        .single()
        .coords
        .clone()
        .map(|x| x.get_snapped_coords());
    infores.scale = projection.single().scale;
    infores.toolst = toolst.get().clone();

    let mut text = text.single_mut();
    let text = &mut text.sections[0].value;
    *text = infores.line();

    if let Some(Ok(spid)) = infores.picked.map(|e| qid.get(e).map(|op| op.get_spid())) {
        text.push_str(&spid);
    }
}
