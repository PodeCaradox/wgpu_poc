use cgmath::Vector2;
use rand::{Rng, thread_rng};
use wgpu::{BindGroup, BindGroupLayout, Device};
use wgpu::util::DeviceExt;

use crate::components::cs_render::shader_types::tile_instance::{AtlasCoordinate, TileInstance};
use crate::components::cs_render::world_render_pipline;

pub const SIZE: i32 = 200;
pub const TILE_SIZE: Vector2<f32> = Vector2::new(32.0, 16.0);
pub const TILE_SIZE_HALF: Vector2<f32> = Vector2::new(16.0, 8.0);


pub(crate) struct Map {
    pub(crate) tiles: Vec<TileInstance>,
}

pub(crate) fn generate_instances() -> Map {
    let mut rng = thread_rng();
    let mut instances: Vec<TileInstance> = Vec::new();
    for y in 0..SIZE {
        for x in 0..SIZE {
            let pos = map_to_screen_tile_pos(Vector2::new(x as f32, y as f32));
            let z = (y as f32 * SIZE as f32 + x as f32) / (SIZE * SIZE) as f32;
            instances.push(
                TileInstance {
                    position: [pos.x, pos.y, 1.0 - z],
                    atlas_coordinate: AtlasCoordinate { coordinate: [rng.gen_range(0..28), 0], index: 0 },
                }
            );
        }
    }
    Map { tiles: instances }
}


pub fn map_to_screen_tile_pos(position: Vector2<f32>) -> Vector2<f32> {
    let position_x = TILE_SIZE_HALF.x * position.x - TILE_SIZE_HALF.x * position.y;
    let position_y = TILE_SIZE_HALF.y * position.x + TILE_SIZE_HALF.y * position.y + TILE_SIZE_HALF.y;
    Vector2::new(position_x, position_y)
}

pub fn map_to_screen_pos_centered(position: Vector2<f32>) -> Vector2<f32> {
    let position_x_centered = TILE_SIZE_HALF.x * position.x - TILE_SIZE_HALF.x * position.y;
    let position_y_centered = TILE_SIZE_HALF.y * position.x + TILE_SIZE_HALF.y * position.y;
    Vector2::new(position_x_centered, position_y_centered)
}

pub fn screen_to_map_pos(position: Vector2<f32>) -> Vector2<i32> {
    let x = (position.y / TILE_SIZE.y) + (position.x / TILE_SIZE.x);
    let y = (position.y / TILE_SIZE.y) - (position.x / TILE_SIZE.x);
    Vector2::new(x as i32, y as i32)
}