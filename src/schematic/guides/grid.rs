use bevy::ecs::component::Component;
use lyon_tessellation::path::Path;

use crate::bevyon::TessInData;

/*
grid guides: major, minor, optional grid spacing / visibility
*/

#[derive(Component)]
struct Grid {
    step: f32,
    style: Path,
}
