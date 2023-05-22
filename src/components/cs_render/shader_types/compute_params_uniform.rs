use bevy_ecs::schedule::Schedule;
use bevy_ecs::system::{Res, ResMut, Resource};
use bevy_ecs::world::World;
use log::error;

use crate::components::cs_util::camera::CustomCamera;
use crate::components::cs_world::map::SIZE;

pub const COMPUTEGROUPSIZE: i32 = 16;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Resource)]
pub struct ComputeParamsUniform {
    pub start_pos: [i32; 2],
    pub map_size: [i32; 2],
    pub columns: i32,
    pub rows: i32,
}

impl ComputeParamsUniform {
    pub(crate) fn new(camera: &CustomCamera, world: &mut World, schedule: &mut Schedule) -> Self {
        let compute_params_uniform = Self {
            map_size: [SIZE, SIZE],
            columns: camera.visible_area.w + (COMPUTEGROUPSIZE - camera.visible_area.w % COMPUTEGROUPSIZE),
            start_pos: [camera.visible_area.x, camera.visible_area.y],
            rows: camera.visible_area.z + (COMPUTEGROUPSIZE - camera.visible_area.z % COMPUTEGROUPSIZE),
        };

        world.insert_resource(compute_params_uniform);
        schedule.add_system(update_compute_params);
        compute_params_uniform
    }
}

pub fn update_compute_params(mut compute_camera_uniform: ResMut<ComputeParamsUniform>, camera: Res<CustomCamera>) {
    compute_camera_uniform.map_size = [SIZE, SIZE];
    compute_camera_uniform.columns = camera.visible_area.w + (COMPUTEGROUPSIZE - camera.visible_area.w % COMPUTEGROUPSIZE);
    compute_camera_uniform.rows = (camera.visible_area.z + (COMPUTEGROUPSIZE - camera.visible_area.z % COMPUTEGROUPSIZE))  / 2;
    compute_camera_uniform.start_pos = [camera.visible_area.x, camera.visible_area.y];
}