//! Device: defines circuit devices such as resistor, mos, etc.
//!
//! a device is comprised of its graphics, bounding box, ports
//!
//! DeviceType held as asset, create mesh asset if instanced at least once
//! update mesh asset whenever projection scale changes
//! for now, all device types are always loaded
//!
//! device ports are jank until bevy ecs relations
//! show ports on device mesh
//! devicetypes to keep track of list of ports and offsets
//! manually make sure ports visual (mesh) and internals (in device types) match

use std::{iter, sync::Arc};

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use euclid::{default::Point2D, Angle, Vector2D};
use lyon_tessellation::{StrokeOptions, VertexBuffers};

use crate::{
    bevyon::{self, build_mesh, stroke, StrokeTessellator},
    schematic::{guides::SchematicCursor, material::SchematicMaterial},
};

use super::{
    readable_idgen::IdTracker, spid, ElementsRes, Pickable, PickableDevice, Preview,
    SchematicElement, SpId,
};

#[derive(Resource)]
pub struct DefaultDevices {
    v: DeviceType,
    // i: DeviceType0,
    r: DeviceType,
}

impl DefaultDevices {
    pub fn voltage_source(&self) -> DeviceType {
        self.v.clone()
    }

    pub fn resistor(&self) -> DeviceType {
        self.r.clone()
    }
}

impl FromWorld for DefaultDevices {
    fn from_world(world: &mut World) -> Self {
        DefaultDevices {
            v: DeviceType::type_v(world),
            r: DeviceType::type_r(world),
        }
    }
}

#[derive(Event, Clone)]
pub struct DeviceType {
    spice_type: &'static str,
    visuals: Mesh2dHandle,
    collider: Arc<dyn Pickable + Send + Sync + 'static>, // schematic element
    ports: Arc<[IVec2]>,                                 // offset of each port
}

impl DeviceType {
    fn type_v(world: &mut World) -> Self {
        let devicecolor = Color::GREEN.rgba_linear_to_vec4();
        let mut stroke_tess = world.resource_mut::<StrokeTessellator>();
        let mut path_builder = bevyon::path_builder().with_svg();
        let r = 1.2;
        path_builder.move_to(Point2D::new(0.0, -3.0));
        path_builder.line_to(Point2D::new(0.0, -r));
        path_builder.move_to(Point2D::new(0.0, 3.0));
        path_builder.line_to(Point2D::new(0.0, r));
        path_builder.move_to(Point2D::new(0.0, 1.0));
        path_builder.line_to(Point2D::new(0.0, 0.2));
        path_builder.move_to(Point2D::new(-0.4, 0.6));
        path_builder.line_to(Point2D::new(0.4, 0.6));
        path_builder.move_to(Point2D::new(-0.4, -0.6));
        path_builder.line_to(Point2D::new(0.4, -0.6));
        path_builder.move_to(Point2D::new(0.0, -r));
        path_builder.arc(
            Point2D::zero(),
            Vector2D::splat(r),
            Angle::two_pi(),
            Angle::zero(),
        );
        let path = path_builder.build();
        let mut buffers = VertexBuffers::new();
        stroke(
            &mut *stroke_tess,
            &path,
            &StrokeOptions::DEFAULT
                .with_line_width(0.1)
                .with_tolerance(0.01)
                .with_line_cap(lyon_tessellation::LineCap::Round),
            &mut buffers,
        );
        let res_mesh = build_mesh(&buffers).with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![devicecolor; buffers.vertices.len()],
        );
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh_res = meshes.add(res_mesh);

        let collider = Arc::new(PickableDevice::_4x6());

        let ports = Arc::new([IVec2::new(0, 3), IVec2::new(0, -3)]);
        DeviceType {
            spice_type: &spid::V,
            visuals: Mesh2dHandle(mesh_res),
            collider,
            ports,
        }
    }
    fn type_r(world: &mut World) -> Self {
        let devicecolor = Color::GREEN.rgba_linear_to_vec4();
        let mut stroke_tess = world.resource_mut::<StrokeTessellator>();
        let mut path_builder = bevyon::path_builder().with_svg();
        path_builder.move_to(Point2D::new(1.00, -0.25));
        path_builder.line_to(Point2D::new(-1.00, -0.75));
        path_builder.move_to(Point2D::new(-1.00, -0.75));
        path_builder.line_to(Point2D::new(1.00, -1.25));
        path_builder.move_to(Point2D::new(1.00, -1.25));
        path_builder.line_to(Point2D::new(-1.00, -1.75));
        path_builder.move_to(Point2D::new(0.00, -2.00));
        path_builder.line_to(Point2D::new(0.00, -3.00));
        path_builder.move_to(Point2D::new(-1.00, -1.75));
        path_builder.line_to(Point2D::new(0.00, -2.00));
        path_builder.move_to(Point2D::new(1.00, 1.75));
        path_builder.line_to(Point2D::new(-1.00, 1.25));
        path_builder.move_to(Point2D::new(1.00, 0.75));
        path_builder.line_to(Point2D::new(-1.00, 0.25));
        path_builder.move_to(Point2D::new(-1.00, 1.25));
        path_builder.line_to(Point2D::new(1.00, 0.75));
        path_builder.move_to(Point2D::new(0.00, 3.00));
        path_builder.line_to(Point2D::new(0.00, 2.00));
        path_builder.move_to(Point2D::new(0.00, 2.00));
        path_builder.line_to(Point2D::new(1.00, 1.75));
        path_builder.move_to(Point2D::new(-1.00, 0.25));
        path_builder.line_to(Point2D::new(1.00, -0.25));
        let path = path_builder.build();
        let mut buffers = VertexBuffers::new();
        stroke(
            &mut *stroke_tess,
            &path,
            &StrokeOptions::DEFAULT
                .with_line_width(0.1)
                .with_tolerance(0.01)
                .with_line_cap(lyon_tessellation::LineCap::Round),
            &mut buffers,
        );
        let res_mesh = build_mesh(&buffers).with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![devicecolor; buffers.vertices.len()],
        );
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh_res = meshes.add(res_mesh);

        let collider = Arc::new(PickableDevice::_4x6());

        let ports = Arc::new([IVec2::new(0, 3), IVec2::new(0, -3)]);

        DeviceType {
            spice_type: &spid::R,
            visuals: Mesh2dHandle(mesh_res),
            collider,
            ports,
        }
    }
}

#[derive(Component)]
struct Port {
    parent_device: Entity,
    offset: IVec2,
}

#[derive(Bundle)]
struct PortBundle {
    port: Port,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
}

impl PortBundle {
    fn new(deviceid: Entity, offset: IVec2, eres: &ElementsRes) -> Self {
        PortBundle {
            port: Port {
                parent_device: deviceid,
                offset,
            },
            mat: MaterialMesh2dBundle {
                mesh: bevy::sprite::Mesh2dHandle(eres.mesh_port.clone()), // TODO create a mesh for port
                material: eres.mat_dflt.clone(),
                ..Default::default()
            },
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Device {
    ports: Vec<Entity>,
}

#[derive(Component, Reflect, Deref, Debug)]
#[reflect(Component)]
struct SpType(&'static str);

#[derive(Bundle)]
struct DeviceBundle {
    device: Device,
    sptype: SpType,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
    se: SchematicElement,
}

impl DeviceBundle {
    fn from_type(dtype: &DeviceType, eres: &ElementsRes, ports: Vec<Entity>) -> Self {
        Self {
            device: Device { ports },
            sptype: SpType(dtype.spice_type),
            mat: MaterialMesh2dBundle {
                mesh: dtype.visuals.clone(),
                material: eres.mat_dflt.clone(),
                ..Default::default()
            },
            se: SchematicElement {
                behavior: dtype.collider.clone(),
            },
        }
    }
}

pub fn spawn_preview_device_from_type(
    mut e: EventReader<DeviceType>,
    mut commands: Commands,
    eres: Res<ElementsRes>,
    cursor: Query<Entity, With<SchematicCursor>>,
) {
    let Some(newtype) = e.read().last() else {
        return;
    };
    let device_entity = commands.spawn_empty().id();

    let ports_entities = newtype
        .ports
        .iter()
        .map(|_| commands.spawn_empty().id())
        .collect::<Vec<Entity>>();
    let device_bundle = (
        DeviceBundle::from_type(newtype, &eres, ports_entities.clone()),
        Preview,
    );
    let port_iter = newtype
        .ports
        .iter()
        .map(|&offset| PortBundle::new(device_entity, offset, &eres))
        .collect::<Vec<PortBundle>>();
    commands.entity(cursor.single()).add_child(device_entity);
    commands.insert_or_spawn_batch(ports_entities.into_iter().zip(port_iter.into_iter()));
    commands.insert_or_spawn_batch(iter::once((device_entity, device_bundle)));
}

fn update_port_location(
    q: Query<(&GlobalTransform, &Device)>,
    mut q_p: Query<(Entity, &mut Transform, &Port)>,
    mut commands: Commands,
) {
    // delete all ports without valid parent device
    for (e, _, port) in q_p.iter() {
        if commands.get_entity(port.parent_device).is_none() {
            commands.entity(e).despawn();
        }
    }
    // update position of ports
    for (gt, d) in q.iter() {
        for pe in d.ports.iter() {
            let Ok((_, mut t, port)) = q_p.get_mut(*pe) else {
                continue;
            };
            t.translation = gt.translation() + port.offset.as_vec2().extend(0.01);
        }
    }
}

fn insert_spid(
    q: Query<(Entity, &SpType), Without<SpId>>,
    mut commands: Commands,
    mut idtracker: ResMut<IdTracker>,
) {
    q.iter().for_each(|(e, sptype)| {
        let spid = match sptype.0 {
            spid::R => SpId::new(spid::R, idtracker.new_r_id("")),
            spid::V => SpId::new(spid::V, idtracker.new_v_id("")),
            _ => {
                panic!("received unknown type {:?}", sptype)
            }
        };
        commands.entity(e).insert(spid);
    });
}

pub struct DevicesPlugin;

impl Plugin for DevicesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_port_location,
                insert_spid,
                spawn_preview_device_from_type,
            ),
        );
    }
}
