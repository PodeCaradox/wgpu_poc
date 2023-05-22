use bevy_ecs::schedule::Schedule;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device};
use wgpu::util::DeviceExt;
use crate::components::cs_render::shader_types::camera_uniform::CameraUniform;
use bevy_ecs::system::{Res, ResMut, Resource};
use bevy_ecs::world::World;
use crate::main_loop::Render;

#[derive(Resource)]
pub struct CameraBinding{
    pub camera_buffer: Buffer,
    pub camera_bind_group: BindGroup,
}


impl CameraBinding {
    pub fn new(device: &Device, camera_uniform: &CameraUniform, world: &mut World, schedule: &mut Schedule) ->BindGroupLayout {
        let camera_buffer = create_camera_buffer(&device, &camera_uniform);
        let camera_bind_group_layout = create_camera_bind_group_layout(&device);
        let camera_bind_group = create_camera_bind_group(&device, &camera_buffer, &camera_bind_group_layout);
        let camera_binding = Self {
            camera_buffer, camera_bind_group
        };
        world.insert_resource(camera_binding);
        schedule.add_system(update_camera_buffer);
        camera_bind_group_layout
    }
}

pub fn update_camera_buffer(render: Res<Render>, camera_uniform: Res<CameraUniform>, camera_binding: Res<CameraBinding>){
    render.queue.write_buffer(&camera_binding.camera_buffer, 0, bytemuck::cast_slice(&[*camera_uniform]));
}

pub fn create_camera_buffer(device: &Device, camera_uniform: &CameraUniform) -> Buffer {
    device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[*camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    )
}

pub fn create_camera_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        ],
        label: Some("camera_bind_group_layout"),
    })
}

pub fn create_camera_bind_group(device: &Device, camera_buffer: &Buffer, camera_bind_group_layout: &BindGroupLayout) -> BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }
        ],
        label: Some("camera_bind_group"),
    })
}
