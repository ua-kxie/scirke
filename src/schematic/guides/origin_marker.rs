use std::f32::INFINITY;

/*
marker that indicates the location of the origin if in view or direction if not
*/
use bevy::{
    math::Vec3A, prelude::*, render::camera::CameraProjection, sprite::MaterialMesh2dBundle,
};

use lyon_tessellation::geom::euclid::{Box2D, Point2D};

use crate::{
    bevyon::{self, CompositeMeshData, TessInData},
    schematic::{camera::SchematicCamera, material::SchematicMaterial},
};

use super::ZoomInvariant;

const Z_DEPTH: f32 = -0.8;

#[derive(Component)]
pub struct OriginMarker;

#[derive(Bundle)]
struct OriginBundle {
    tess_data: CompositeMeshData,
    mat_bundle: MaterialMesh2dBundle<SchematicMaterial>,
    origin_marker: OriginMarker,
    zoom_invariant_marker: ZoomInvariant,
}

pub fn setup(mut commands: Commands, mut materials: ResMut<Assets<SchematicMaterial>>) {
    let mut path_builder = bevyon::path_builder();

    let osize = 20.0;
    let ssize = 4.0;
    path_builder.begin(Point2D::new(0.0, -osize));
    path_builder.line_to(Point2D::new(0.0, osize));
    path_builder.end(false);
    path_builder.begin(Point2D::new(-osize, 0.0));
    path_builder.line_to(Point2D::new(osize, 0.0));
    path_builder.end(false);
    path_builder.add_rectangle(
        &Box2D {
            min: Point2D::splat(-ssize),
            max: Point2D::splat(ssize),
        },
        lyon_tessellation::path::Winding::Positive,
    );

    let path = Some(path_builder.build());

    let tessellator_input_data = TessInData {
        path,
        stroke: Some(
            bevyon::StrokeOptions::DEFAULT
                .with_line_width(2.0)
                .with_tolerance(1.0),
        ),
        fill: None,
    };
    commands.spawn(OriginBundle {
        tess_data: CompositeMeshData::from_single_w_color(
            tessellator_input_data,
            bevy::prelude::Color::Srgba(Srgba::hex("4682B4").unwrap()),
        ),
        mat_bundle: MaterialMesh2dBundle {
            material: materials.add(SchematicMaterial {
                color: Color::BLACK.with_alpha(0.0).into(),
            }),
            transform: Transform::from_translation(Vec3::Z * Z_DEPTH),
            ..Default::default()
        },
        origin_marker: OriginMarker,
        zoom_invariant_marker: ZoomInvariant,
    });
}

/// this system changes origin scale so its size appear independent of camera zoom,
/// and keeps it visible along edge of screen if origin is not in view
pub fn main(
    ce: Query<(Entity, &OrthographicProjection), With<SchematicCamera>>,
    mut params: ParamSet<(TransformHelper, Query<&mut Transform, With<OriginMarker>>)>,
) {
    let (ent, proj) = ce.single();
    let frustum = proj.compute_frustum(&params.p0().compute_global_transform(ent).unwrap());
    let mut translation = Vec3A::ZERO;
    for hs in frustum.half_spaces {
        // accumulate vector to move origin onto point if distance is negative
        translation -= hs.normal() * hs.d().clamp(-INFINITY, 0.0);
    }

    let mut qt = params.p1();
    let mut transform = qt.single_mut();
    *transform = transform.with_translation(translation.into());
}
