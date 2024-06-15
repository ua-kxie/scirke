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
use euclid::default::{Box2D, Point2D};
use lyon_tessellation::{FillOptions, StrokeOptions, VertexBuffers};

use crate::{
    bevyon::{self, build_mesh, fill, stroke, FillTessellator, StrokeTessellator},
    schematic::{guides::SchematicCursor, material::SchematicMaterial},
};

use super::{
    readable_idgen::IdTracker, spid, ElementsRes, Pickable, PickableDevice, Preview,
    SchematicElement, SpId,
};

#[derive(Resource)]
pub struct DefaultDevices {
    v: DeviceType0,
    // i: DeviceType0,
    r: DeviceType0,
}

impl DefaultDevices {
    pub fn voltage_source(&self) -> DeviceType0 {
        self.v.clone()
    }

    pub fn resistor(&self) -> DeviceType0 {
        self.r.clone()
    }
}

impl FromWorld for DefaultDevices {
    fn from_world(world: &mut World) -> Self {
        DefaultDevices {
            v: DeviceType0::type_v(world),
            r: DeviceType0::type_r(world),
        }
    }
}

#[derive(Event, Clone)]
pub struct DeviceType0 {
    visuals: Mesh2dHandle,
    collider: Arc<dyn Pickable + Send + Sync + 'static>, // schematic element
    ports: Arc<[IVec2]>,                                 // offset of each port
}

impl DeviceType0 {
    fn type_v(world: &mut World) -> Self {
        let devicecolor = Color::GREEN.rgba_linear_to_vec4();
        let mut stroke_tess = world.resource_mut::<StrokeTessellator>();
        let mut path_builder = bevyon::path_builder();
        path_builder.begin(Point2D::new(0.0, -3.0));
        path_builder.line_to(Point2D::new(0.0, 3.0));
        path_builder.end(true);
        let path = path_builder.build();
        let mut buffers = VertexBuffers::new();
        // fill(&mut *fill_tess, &path, &FillOptions::DEFAULT, &mut buffers);
        stroke(
            &mut *stroke_tess,
            &path,
            &StrokeOptions::DEFAULT,
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
        DeviceType0 {
            visuals: Mesh2dHandle(mesh_res),
            collider,
            ports,
        }
    }
    fn type_r(world: &mut World) -> Self {
        let devicecolor = Color::GREEN.rgba_linear_to_vec4();
        let mut fill_tess = world.resource_mut::<FillTessellator>();
        let mut path_builder = bevyon::path_builder();
        path_builder.add_rectangle(
            &Box2D {
                min: Point2D::new(-2.0, -3.0),
                max: Point2D::new(2.0, 3.0),
            },
            lyon_tessellation::path::Winding::Positive,
        );
        let path = path_builder.build();
        let mut buffers = VertexBuffers::new();
        fill(&mut *fill_tess, &path, &FillOptions::DEFAULT, &mut buffers);
        let res_mesh = build_mesh(&buffers).with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![devicecolor; buffers.vertices.len()],
        );
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh_res = meshes.add(res_mesh);

        let collider = Arc::new(PickableDevice::_4x6());

        let ports = Arc::new([IVec2::new(0, 3), IVec2::new(0, -3)]);

        DeviceType0 {
            visuals: Mesh2dHandle(mesh_res),
            collider,
            ports,
        }
    }
}

#[derive(Component)]
struct Port0 {
    parent_device: Entity,
    offset: IVec2,
}

#[derive(Bundle)]
struct PortBundle0 {
    port: Port0,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
}

impl PortBundle0 {
    fn new(deviceid: Entity, offset: IVec2, eres: &ElementsRes) -> Self {
        PortBundle0 {
            port: Port0 {
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

#[derive(Component)]
struct Device0 {
    ports: Vec<Entity>,
}

#[derive(Bundle)]
struct DeviceBundle0 {
    device: Device0,
    mat: MaterialMesh2dBundle<SchematicMaterial>,
    se: SchematicElement,
}

impl DeviceBundle0 {
    fn from_type(dtype: &DeviceType0, eres: &ElementsRes, ports: Vec<Entity>) -> Self {
        Self {
            device: Device0 { ports },
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
    mut e: EventReader<DeviceType0>,
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
        DeviceBundle0::from_type(newtype, &eres, ports_entities.clone()),
        Preview,
    );
    let port_iter = newtype
        .ports
        .iter()
        .map(|&offset| PortBundle0::new(device_entity, offset, &eres))
        .collect::<Vec<PortBundle0>>();
    commands.entity(cursor.single()).add_child(device_entity);
    commands.insert_or_spawn_batch(ports_entities.into_iter().zip(port_iter.into_iter()));
    commands.insert_or_spawn_batch(iter::once((device_entity, device_bundle)));
}

fn update_port_location(
    q: Query<(&GlobalTransform, &Device0)>,
    mut q_p: Query<(Entity, &mut Transform, &Port0)>,
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
    q: Query<Entity, (With<Device0>, Without<SpId>)>,
    mut commands: Commands,
    mut idtracker: ResMut<IdTracker>,
) {
    q.iter().for_each(|e| {
        commands
            .entity(e)
            .insert(SpId::new(spid::R, idtracker.new_r_id("")));
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
