use bevy_ecs::prelude::{Schedule, World};
use bevy_ecs::system::{Res, ResMut, Resource};
use cgmath::{Matrix4, SquareMatrix, Vector2, Vector4, Zero};
use winit::event::VirtualKeyCode;

use crate::components::cs_util::input::Input;
use crate::components::cs_world::map;
use crate::components::cs_world::map::{screen_to_map_pos, TILE_SIZE};

const MAX_ZOOM_OUT: f32 = 0.125_f32;
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(Debug, Clone, Resource, Copy)]
pub struct CustomCamera {
    speed: f32,
    position: Vector2<f32>,
    zoom: f32,
    pub size: Vector2<f32>,
    pub visible_area: Vector4<i32>,
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    changed: bool,
}

impl CustomCamera {
    pub fn new(pos: Vector2<f32>, screen_size: Vector2<f32>, world: &mut World, schedule: &mut Schedule) -> Self {
        let world_pos = map::map_to_screen_pos_centered(pos);
        let mut camera = Self {
            speed: 1.0_f32,
            position: world_pos,
            zoom: 1.0_f32,
            size: screen_size,
            visible_area: Vector4::new(0, 0, 0, 0),
            view: calculate_view_matrix(1.0, world_pos, screen_size),
            projection: Self::calculate_proj_matrix(screen_size),
            changed: true,
        };
        update_visible_ara(&mut camera);
        world.insert_resource(camera);
        schedule.add_system(update_input);
        camera
    }

    pub fn update_projection(&mut self, size: Vector2<f32>) {
        self.projection = Self::calculate_proj_matrix(size);
        self.size = size;
        self.view = calculate_view_matrix(self.zoom, self.position, self.size);

        update_visible_ara(self);
    }

    fn calculate_proj_matrix(screen_size: Vector2<f32>) -> Matrix4<f32> {
        create_orthographic_off_center(0.0, screen_size.x as f32, screen_size.y as f32, 0.0, 0.0, -1.0)
    }
}

fn create_orthographic_off_center(left: f32, right: f32, bottom: f32, top: f32, z_near_plane: f32, z_far_plane: f32) -> Matrix4<f32> {
    let two: f64 = 2.0;

    let c0r0 = (two / (right as f64 - left as f64)) as f32;
    let c0r1 = 0.0;
    let c0r2 = 0.0;
    let c0r3 = 0.0;

    let c1r0 = 0.0;
    let c1r1 = (two / (top as f64 - bottom as f64)) as f32;
    let c1r2 = 0.0;
    let c1r3 = 0.0;

    let c2r0 = 0.0;
    let c2r1 = 0.0;
    let c2r2 = (1.0 as f64 / (z_near_plane as f64 - z_far_plane as f64)) as f32;
    let c2r3 = 0.0;

    let c3r0 = ((left as f64 + right as f64) / (left as f64 - right as f64)) as f32;
    let c3r1 = ((top as f64 + bottom as f64) / (bottom as f64 - top as f64)) as f32;
    let c3r2 = ((z_near_plane as f64) / (z_near_plane as f64 - z_far_plane as f64)) as f32;
    let c3r3 = 1.0;

    #[cfg_attr(rustfmt, rustfmt_skip)]
    Matrix4::new(
        c0r0, c0r1, c0r2, c0r3,
        c1r0, c1r1, c1r2, c1r3,
        c2r0, c2r1, c2r2, c2r3,
        c3r0, c3r1, c3r2, c3r3,
    )
}

pub fn update_input(
    mut camera: ResMut<CustomCamera>,
    //time: Res<Time>,
    //mut ev_scroll: EventReader<MouseWheel>,
    keyboard_input: Res<Input<VirtualKeyCode>>,
) {
    let current_speed: f32 = camera.speed;
    camera.changed |= check_input(
        &mut camera.position.y,
        VirtualKeyCode::W,
        -current_speed,
        1.0,
        &keyboard_input,
    );
    camera.changed |= check_input(
        &mut camera.position.y,
        VirtualKeyCode::S,
        current_speed,
        1.0,
        &keyboard_input,
    );
    camera.changed |= check_input(
        &mut camera.position.x,
        VirtualKeyCode::D,
        current_speed,
        1.0,
        &keyboard_input,
    );
    camera.changed |= check_input(
        &mut camera.position.x,
        VirtualKeyCode::A,
        -current_speed,
        1.0,
        &keyboard_input,
    );

    if keyboard_input.just_pressed(VirtualKeyCode::NumpadSubtract) {
        camera.zoom /= 2.0;
        camera.changed = true;
        adjust_zoom(&mut camera.zoom);
    } else if keyboard_input.just_pressed(VirtualKeyCode::NumpadAdd) {
        camera.zoom *= 2.0;
        camera.changed = true;
        adjust_zoom(&mut camera.zoom);
    }

    /*
    for ev in ev_scroll.iter() {
        if ev.y < 0.0 {
            camera.zoom /= 2.0;
            camera.changed = true;
            adjust_zoom(&mut camera.zoom);
        } else if ev.y > 0.0 {
            camera.zoom *= 2.0;
            camera.changed = true;
            adjust_zoom(&mut camera.zoom);
        }
    }
    */

    if camera.changed {
        camera.changed = false;
        update_matrix(&mut camera);
    }
}

fn calculate_view_matrix(zoom: f32, pos: Vector2<f32>, screen_offset: Vector2<f32>) -> Matrix4<f32> {
    Matrix4::from_translation(((screen_offset.x * 0.5).into(), (screen_offset.y * 0.5).into(), 0.0).into()) *
    Matrix4::from_nonuniform_scale(zoom, zoom, 1.0) *
    Matrix4::from_translation((-pos.x, -pos.y, 0.0).into())
}

fn check_input(
    camera_position: &mut f32,
    key: VirtualKeyCode,
    camera_speed: f32,
    time_delta: f32,
    keyboard_input: &Res<Input<VirtualKeyCode>>,
) -> bool {
    if !keyboard_input.pressed(key) {
        return false;
    }
    *camera_position += camera_speed * time_delta * 10.0;
    true
}


fn adjust_zoom(zoom: &mut f32) {
    if *zoom < MAX_ZOOM_OUT {
        *zoom = MAX_ZOOM_OUT;
    } else if *zoom > 6.4_f32 {
        *zoom = 6.4_f32;
    }
}

fn update_matrix(camera: &mut CustomCamera) {
    camera.view = calculate_view_matrix(camera.zoom, camera.position, camera.size);
    update_visible_ara(camera);
}


fn update_visible_ara(camera: &mut CustomCamera) {
    let inverse_matrix = camera.view.invert().unwrap();
    let tl = transform(Vector2::zero(), inverse_matrix);
    let tr = transform(Vector2::zero(), inverse_matrix);
    let bl = transform(Vector2::zero(), inverse_matrix);
    let br = transform(Vector2::new(camera.size.x, camera.size.y), inverse_matrix);
    let min = Vector2::new(
        tl.x.min(tr.x.min(bl.x.min(br.x))),
        tl.y.min(tr.y.min(bl.y.min(br.y))));
    let mut max = Vector2::new(
        tl.x.max(tr.x.max(bl.x.max(br.x))),
        tl.y.max(tr.y.max(bl.y.max(br.y))));


    max = max - min;

    let pos = screen_to_map_pos(Vector2::new(min.x + max.x, min.y));
    let size = Vector2::new(max.x / TILE_SIZE.x * 2.0, max.y / TILE_SIZE.y);// *2 because rows differnce
    camera.visible_area = Vector4::new(pos.x as i32 - 4, pos.y as i32 - 4, size.x as i32 + 8, size.y as i32 + 8);
}

pub fn transform(position: Vector2<f32>, matrix: Matrix4<f32>) -> Vector2<f32> {
    let x = position.x * matrix.x.x + position.y * matrix.y.x + matrix.w.x;
    let y = position.x * matrix.x.y + position.y * matrix.y.y + matrix.w.y;
    Vector2::new(x, y)
}
