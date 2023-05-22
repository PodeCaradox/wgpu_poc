use bevy_ecs::prelude::{Schedule, World};
use bevy_ecs::system::{Res, ResMut, Resource};
use wgpu::{Buffer, Device};
use wgpu::util::DeviceExt;
use crate::components::cs_render::shader_types::compute_params_uniform::ComputeParamsUniform;
use crate::components::cs_util::camera::CustomCamera;
use crate::components::cs_world::map::SIZE;


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Resource)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub(crate) fn new(camera: &CustomCamera, world: &mut World, schedule: &mut Schedule) -> Self {
        let camera_uniform = Self {
            view_proj: (camera.projection * camera.view).into()
        };

        world.insert_resource(camera_uniform);
        schedule.add_system(update_view_proj);
        camera_uniform
    }
}



pub fn update_view_proj(mut camera_uniform: ResMut<CameraUniform>, camera: Res<CustomCamera>) {
    camera_uniform.view_proj = (camera.projection * camera.view).into();
}