use bevy::{
    ecs::{component::Component, system::Query},
    prelude::*,
    render::render_asset::RenderAssetUsages,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use lyon_tessellation::{geom::Point, FillOptions};

use crate::{
    bevyon::{
        self, build_mesh_im, CompositeMeshData, EmptyMesh, FillTessellator, StrokeTessellator,
        TessInData,
    },
    schematic::{camera::SchematicCamera, material::SchematicMaterial, SnapSet},
};

/*
grid guides: major, minor, optional grid spacing / visibility
*/

/// struct containing data for building a homogenous grid (one spacing, color)
struct HomoGrid {
    step: f32,
    mesh_hndl: Mesh2dHandle,
}

#[derive(Component, Deref, DerefMut)]
struct Grid(Vec<HomoGrid>);

#[derive(Bundle)]
struct GridBundle {
    // tess_data: CompositeMeshData,
    mat_bundle: MaterialMesh2dBundle<SchematicMaterial>,
    marker: Grid,
}

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(PostUpdate, build_grid.in_set(SnapSet));
    }
}

const Z_DEPTH: f32 = -0.8;

fn major(
    empty: Mesh,
    meshes: &mut ResMut<Assets<Mesh>>,
    fill_tess: &mut ResMut<FillTessellator>,
    stroke_tess: &mut ResMut<StrokeTessellator>,
) -> HomoGrid {
    let mut path_builder = bevyon::path_builder();
    path_builder.add_circle(
        Point::zero(),
        1.0,
        lyon_tessellation::path::Winding::Positive,
    );
    let path = Some(path_builder.build());

    let data = CompositeMeshData::from_single_w_color(
        TessInData {
            path,
            stroke: None,
            fill: Some(FillOptions::DEFAULT),
        },
        Color::hex("4682B4").unwrap(),
    );
    let mut marker_mesh = build_mesh_im(empty, &data, stroke_tess, fill_tess);
    marker_mesh.asset_usage = RenderAssetUsages::MAIN_WORLD;
    let hndl = Mesh2dHandle(meshes.add(marker_mesh));
    HomoGrid {
        step: 16.0,
        mesh_hndl: hndl,
    }
}

fn minor(
    empty: Mesh,
    meshes: &mut ResMut<Assets<Mesh>>,
    fill_tess: &mut ResMut<FillTessellator>,
    stroke_tess: &mut ResMut<StrokeTessellator>,
) -> HomoGrid {
    let mut path_builder = bevyon::path_builder();
    path_builder.add_circle(
        Point::zero(),
        0.5,
        lyon_tessellation::path::Winding::Positive,
    );
    let path = Some(path_builder.build());

    let data = CompositeMeshData::from_single_w_color(
        TessInData {
            path,
            stroke: None,
            fill: Some(FillOptions::DEFAULT),
        },
        Color::hex("468200").unwrap(),
    );
    let mut marker_mesh = build_mesh_im(empty, &data, stroke_tess, fill_tess);
    marker_mesh.asset_usage = RenderAssetUsages::MAIN_WORLD;
    let hndl = Mesh2dHandle(meshes.add(marker_mesh));
    HomoGrid {
        step: 2.0,
        mesh_hndl: hndl,
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<SchematicMaterial>>,
    mut fill_tess: ResMut<FillTessellator>,
    mut stroke_tess: ResMut<StrokeTessellator>,
    empty_mesh: Res<EmptyMesh>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let empty = empty_mesh.0.clone();

    commands.spawn(GridBundle {
        mat_bundle: MaterialMesh2dBundle {
            material: materials.add(SchematicMaterial {
                color: Color::BLACK.with_a(0.0),
            }),
            transform: Transform::from_translation(Vec3::Z * Z_DEPTH),
            ..Default::default()
        },
        marker: Grid(vec![
            major(empty.clone(), &mut meshes, &mut fill_tess, &mut stroke_tess),
            minor(empty, &mut meshes, &mut fill_tess, &mut stroke_tess),
        ]),
    });
}

fn build_grid(
    ce: Query<
        (&GlobalTransform, &OrthographicProjection),
        (
            With<SchematicCamera>,
            Or<(Changed<OrthographicProjection>, Changed<GlobalTransform>)>,
        ),
    >,
    mut q_grid: Query<(&mut Mesh2dHandle, &Grid)>,
    mut meshes: ResMut<Assets<Mesh>>,
    empty_mesh: Res<EmptyMesh>,
    // filter on changed camera global transform, only run this if camera global transform changed
    // only needs to be run if: zoom changed, aabb changed to encompases new grid points, grid appearance changed
    // for now run whenever camera changes, gpu instancing later. Then customizable grid appearance.
) {
    // get screen aabb, zoom scale
    let Ok((cgt, proj)) = ce.get_single() else {
        return;
    };
    let translation = cgt.translation().truncate();
    let mut screen_patch = proj.area;
    screen_patch.min += translation;
    screen_patch.max += translation;
    let scale = proj.scale;

    // get empty mesh to merge stuff into
    let mut mesh = empty_mesh.0.clone();

    let (mut meshhndl, grid) = q_grid.single_mut();
    for homogrid in &grid.0 {
        let cols_count = ((screen_patch.width() / homogrid.step).ceil() + 1.0) as u8;
        let rows_count = ((screen_patch.height() / homogrid.step).ceil() + 1.0) as u8;
        // if row / column exceed 100 - don't bother
        // (would be better to make this based on screen space density)
        if cols_count > 100 || rows_count > 100 {
            continue;
        }
        // seed is top right point marked by current homogrid
        let seed = (screen_patch.min / homogrid.step).round() * homogrid.step;
        // scale marker mesh by zoom scale to counteract camera zoom
        let marker_mesh = meshes
            .get(&homogrid.mesh_hndl.0)
            .unwrap()
            .clone()
            .scaled_by(Vec3::splat(scale));

        for row in 0..rows_count {
            let yofst = row as f32 * homogrid.step;
            for col in 0..cols_count {
                let xofst = col as f32 * homogrid.step;
                let p = seed + Vec2::new(xofst, yofst);
                mesh.merge(marker_mesh.clone().translated_by(p.extend(0.0)));
            }
        }
    }
    meshhndl.0 = meshes.add(mesh)
}
