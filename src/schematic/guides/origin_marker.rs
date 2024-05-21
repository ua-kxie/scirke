use std::f32::INFINITY;

/*
marker that indicates the location of the origin if in view or direction if not
*/
use bevy::{
    math::Vec3A, prelude::*, render::camera::CameraProjection, sprite::MaterialMesh2dBundle,
};

use lyon_tessellation::geom::euclid::{Box2D, Point2D};

use crate::{
    bevyon::{self, TessInData},
    schematic::camera::SchematicCamera,
};

use super::ZoomInvariant;

const Z_DEPTH: f32 = 0.9;

#[derive(Component)]
pub struct OriginMarker;

#[derive(Bundle)]
struct OriginBundle {
    tess_data: TessInData,
    mat_bundle: MaterialMesh2dBundle<ColorMaterial>,
    origin_marker: OriginMarker,
    zoom_invariant_marker: ZoomInvariant,
}

pub fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let mut path_builder = bevyon::path_builder();

    let osize = 50.0;
    let ssize = 10.0;
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
                .with_line_width(5.0)
                .with_tolerance(1.0),
        ),
        fill: None,
        z_depth: Z_DEPTH,
    };
    commands.spawn(OriginBundle {
        tess_data: tessellator_input_data,
        mat_bundle: MaterialMesh2dBundle {
            material: materials.add(Color::WHITE),
            ..Default::default()
        },
        origin_marker: OriginMarker,
        zoom_invariant_marker: ZoomInvariant,
    });
}

/// this system changes origin scale so its size appear independent of camera zoom,
/// and keeps it visible along edge of screen if origin is not in view
pub fn main(
    ce: Query<(&OrthographicProjection, &GlobalTransform), With<SchematicCamera>>,
    mut om_transform: Query<&mut Transform, With<OriginMarker>>,
) {
    let (proj, cgt) = ce.single();
    let mut transform = om_transform.single_mut();

    let frustum = proj.compute_frustum(cgt);
    let mut translation = Vec3A::ZERO;
    for hs in frustum.half_spaces {
        // accumulate vector to move origin onto point if distance is negative
        translation -= hs.normal() * hs.d().clamp(-INFINITY, 0.0);
    }
    *transform = transform
        .with_translation(translation.into())
}
