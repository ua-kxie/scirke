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
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use euclid::{default::Point2D, Angle, Vector2D};
use lyon_tessellation::{StrokeOptions, VertexBuffers};

use crate::{
    bevyon::{self, build_mesh, stroke, StrokeTessellator},
    schematic::{guides::SchematicCursor, material::SchematicMaterial, FreshLoad, SchematicLoaded},
};

use super::{
    readable_idgen::IdTracker, spid, ElementsRes, Pickable, PickableDevice, PickableElement,
    Preview, SchematicElement, Selected, SpDeviceId,
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
    spice_type: spid::SpDeviceType,
    visuals: Mesh2dHandle,
    collider: Arc<dyn Pickable + Send + Sync + 'static>, // schematic element
    ports: Arc<[IVec2]>,                                 // offset of each port
}

impl DeviceType {
    fn as_non_reflect_bundle(&self) -> impl Bundle {
        (
            self.visuals.clone(),
            PickableElement {
                behavior: self.collider.clone(),
            },
        )
    }
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
            spice_type: spid::SpDeviceType::V,
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
            spice_type: spid::SpDeviceType::R,
            visuals: Mesh2dHandle(mesh_res),
            collider,
            ports,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
struct Port {
    parent_device: Entity,
    offset: IVec2,
}
impl MapEntities for Port {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.parent_device = entity_mapper.map_entity(self.parent_device);
    }
}

#[derive(Bundle)]
struct PortBundle {
    port: Port,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
    se: SchematicElement,
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
            se: SchematicElement {
                schtype: spid::SchType::Port,
            },
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
struct Device {
    ports: Vec<Entity>,
}
impl MapEntities for Device {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.ports = self
            .ports
            .iter()
            .map(|e| entity_mapper.map_entity(*e))
            .collect();
    }
}

#[derive(Bundle)]
struct DeviceBundle {
    device: Device,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
    pe: PickableElement,
    se: SchematicElement,
}

impl DeviceBundle {
    fn from_type(dtype: &DeviceType, eres: &ElementsRes, ports: Vec<Entity>) -> Self {
        Self {
            device: Device { ports },
            mat: MaterialMesh2dBundle {
                mesh: dtype.visuals.clone(),
                material: eres.mat_dflt.clone(),
                ..Default::default()
            },
            pe: PickableElement {
                behavior: dtype.collider.clone(),
            },
            se: SchematicElement {
                schtype: spid::SchType::Spice(spid::SpType::Device(dtype.spice_type.clone())),
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
        Selected,
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
    for (device_gt, d) in q.iter() {
        for port_entity in d.ports.iter() {
            let Ok((_, mut port_t, port)) = q_p.get_mut(*port_entity) else {
                continue;
            };
            let mut newt = device_gt.transform_point(port.offset.extend(0).as_vec3());
            newt.z = 0.01;
            port_t.translation = newt;
        }
    }
}

/// inspert spid component for entities which have SpDeviceType but not spid
fn insert_spid(
    q: Query<(Entity, &SchematicElement), (Without<SpDeviceId>, With<Device>)>,
    mut commands: Commands,
    mut idtracker: ResMut<IdTracker>,
) {
    q.iter().for_each(|(e, schtype)| {
        let spid = match schtype.get_dtype().unwrap() {
            spid::SpDeviceType::V => SpDeviceId::new(spid::SpDeviceType::V, idtracker.new_v_id("")),
            spid::SpDeviceType::R => SpDeviceId::new(spid::SpDeviceType::R, idtracker.new_r_id("")),
        };
        commands.entity(e).insert(spid);
    });
}

/// this system iterates through
/// inserts non-refelct components for device type elements
/// useful for applying mesh handles and such after loading
fn insert_non_reflect(
    qd: Query<(Entity, &Device, &SchematicElement), With<FreshLoad>>,
    default_devices: Res<DefaultDevices>,
    eres: Res<ElementsRes>,
    mut commands: Commands,
) {
    for (device_ent, device, spid) in qd.iter() {
        let bundle = match spid.get_dtype().unwrap() {
            spid::SpDeviceType::V => (
                default_devices.v.as_non_reflect_bundle(),
                eres.mat_dflt.clone(),
            ),
            spid::SpDeviceType::R => (
                default_devices.r.as_non_reflect_bundle(),
                eres.mat_dflt.clone(),
            ),
        };
        commands.entity(device_ent).insert(bundle);
        commands.entity(device_ent).remove::<FreshLoad>();

        for port_ent in device.ports.iter() {
            commands.entity(*port_ent).insert((
                eres.mat_dflt.clone(),
                Mesh2dHandle(eres.mesh_port.clone()),
                SchematicElement {
                    schtype: spid::SchType::Port,
                },
            ));
            commands.entity(*port_ent).remove::<FreshLoad>();
        }
    }
}

pub struct DevicesPlugin;

impl Plugin for DevicesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Port>();
        app.register_type::<Device>();
        app.add_systems(
            Update,
            (
                update_port_location,
                insert_spid,
                spawn_preview_device_from_type,
            ),
        );
        app.add_systems(
            PreUpdate,
            insert_non_reflect.run_if(on_event::<SchematicLoaded>()),
        );
        app.register_type::<SpDeviceId>();
        app.add_event::<DeviceType>();
    }
}
