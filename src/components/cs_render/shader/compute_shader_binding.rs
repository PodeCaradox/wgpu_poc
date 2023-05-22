use bevy_ecs::schedule::Schedule;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device};
use wgpu::util::DeviceExt;
use crate::components::cs_render::shader_types::camera_uniform::CameraUniform;
use bevy_ecs::system::{Res, ResMut, Resource};
use bevy_ecs::world::World;
use cgmath::Vector2;
use crate::components::cs_render::shader_types::compute_params_uniform::ComputeParamsUniform;
use crate::main_loop::Render;

#[derive(Resource)]
pub struct ComputeParamsBinding {
    pub compute_shader_buffer: Buffer,
    pub compute_shader_bind_group: BindGroup,
}

impl ComputeParamsBinding {
    pub fn new(device: &Device, compute_camera_uniform: &ComputeParamsUniform, world: &mut World, schedule: &mut Schedule) -> BindGroupLayout {
        let compute_shader_buffer = create_compute_shader_buffer(&device, &compute_camera_uniform);
        let compute_shader_bind_group_layout = create_compute_bind_group_layout(&device);
        let compute_shader_bind_group = create_compute_bind_group(&device, &compute_shader_buffer, &compute_shader_bind_group_layout);
        let compute_shader_binding = Self {
            compute_shader_buffer, compute_shader_bind_group
        };
        world.insert_resource(compute_shader_binding);
        schedule.add_system(update_compute_params_buffer);
        compute_shader_bind_group_layout
    }
}

pub fn update_compute_params_buffer(render: Res<Render>, compute_camera_uniform: Res<ComputeParamsUniform>, compute_shader_binding: Res<ComputeParamsBinding>){
    render.queue.write_buffer(&compute_shader_binding.compute_shader_buffer, 0, bytemuck::cast_slice(&[*compute_camera_uniform]));
}

pub fn create_compute_shader_buffer(device: &Device, compute_camera_uniform: &ComputeParamsUniform) -> Buffer {
    device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Compute Buffer"),
            contents: bytemuck::cast_slice(&[*compute_camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    )
}

pub fn create_compute_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
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

pub fn create_compute_bind_group(device: &Device, compute_shader_buffer: &Buffer, compute_shader_bind_group_layout: &BindGroupLayout) -> BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &compute_shader_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: compute_shader_buffer.as_entire_binding(),
            }
        ],
        label: Some("camera_bind_group"),
    })
}
