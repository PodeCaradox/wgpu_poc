use std::iter;

use bevy_ecs::schedule::Schedule;
use bevy_ecs::system::Res;
use bevy_ecs::world::World;
use cgmath::Vector2;
use log::error;
use log::warn;
use winit::dpi::Size;
use winit::event_loop::ControlFlow;

use crate::components::cs_render::shader::camera_binding::CameraBinding;
use crate::components::cs_render::shader::compute_shader_binding::ComputeParamsBinding;
use crate::components::cs_render::shader_types::compute_params_uniform::{COMPUTEGROUPSIZE, ComputeParamsUniform};
use crate::components::cs_render::shader_types::geometry::VERTICES;
use crate::components::cs_util::cs_window::State;
use crate::components::cs_world::map::SIZE;
use crate::main_loop::{DummyTest, Render};

pub fn render_game_world(world: &mut World, state: &mut State, control_flow: &mut ControlFlow) {
    match render_instances(
        world.get_resource::<Render>().unwrap(),
        world.get_resource::<CameraBinding>().unwrap(),
        world.get_resource::<DummyTest>().unwrap(),
        world.get_resource::<ComputeParamsBinding>().unwrap(),
        world.get_resource::<ComputeParamsUniform>().unwrap(),
    ) {
        Ok(_) => {}
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(world),
        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
        Err(wgpu::SurfaceError::Timeout) => warn!("Surface timeout"),
    }
}

pub fn render_instances(
    render: &Render,
    camera_binding: &CameraBinding,
    dummy_test: &DummyTest,
    compute_params_binding: &ComputeParamsBinding,
    compute_params_uniform: &ComputeParamsUniform,
) -> Result<(), wgpu::SurfaceError> {
    let mut encoder = render.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Life grid step"),
        });
        cpass.set_pipeline(&dummy_test.compute_pipeline);
        cpass.set_bind_group(0, &compute_params_binding.compute_shader_bind_group, &[]);
        cpass.set_bind_group(1, &dummy_test.compute_buffer_bind_group, &[]);
        cpass.set_bind_group(2, &dummy_test.compute_visible_buffer_bind_group, &[]);
        cpass.dispatch_workgroups((compute_params_uniform.columns / COMPUTEGROUPSIZE) as u32, ((compute_params_uniform.rows * 2) / COMPUTEGROUPSIZE) as u32, 1);
    };


    let output = render.surface.get_current_texture().unwrap();
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &render.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        let instances = calc_visible_tiles(compute_params_uniform.start_pos[0], compute_params_uniform.start_pos[1], compute_params_uniform.columns, compute_params_uniform.rows * 2, compute_params_uniform.map_size[0], compute_params_uniform.map_size[1]);
        render_pass.set_pipeline(&dummy_test.render_pipeline);
        render_pass.set_bind_group(0, &dummy_test.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &camera_binding.camera_bind_group, &[]);
        render_pass.set_bind_group(2, &dummy_test.instance_buffer_bind_group, &[]);
        render_pass.set_vertex_buffer(0, dummy_test.geometry_buffer.slice(..));
        render_pass.draw(0..VERTICES.len() as u32, 0..(instances) as u32);
        //error!("Wuff {:?}", instances);
    }

    render.queue.submit(iter::once(encoder.finish()));
    output.present();
    Ok(())
}


fn calc_visible_tiles(start_pos_x: i32, start_pos_y: i32, columns: i32, rows: i32, map_size_x: i32, map_size_y: i32) -> i32 {
    let mut visible_index = 0;
    let mut start = Vector2::new(start_pos_x, start_pos_y);
    let mut outside = 1;

    for i in 0..columns {
        start.x += 1;
        start.y += 1;
        if start.x >= 0 && start.y >= 0 && start.y < map_size_y && start.x < map_size_x {
            outside = 0;
            break;
        }
    }

    if outside == 1 {
        if start_pos_x + start_pos_y < map_size_x {
            let mut left = Vector2::new(start_pos_x - rows, start_pos_y + rows);
            left.x += left.y;
            left.y -= left.y;

            let mut right_bottom_screen = Vector2::new(start_pos_x + columns, start_pos_y + columns);

            if right_bottom_screen.x + right_bottom_screen.y > map_size_x {
                start = Vector2::new(map_size_x - 1, 0);
            } else {
                right_bottom_screen.x += right_bottom_screen.y;
                right_bottom_screen.y -= right_bottom_screen.y;
                start = right_bottom_screen;
            }

            let mut difference = start.x - left.x;
            difference += difference % 2;
            difference /= 2;
            start.x -= difference;
            start.y -= difference;
        } else {
            let to_the_left = start_pos_x - map_size_x;
            start = Vector2::new(start_pos_x - to_the_left, start_pos_y + to_the_left);
        }
    } else {
        start = Vector2::new(start_pos_x, start_pos_y);
    }

    for i in 0..rows {
        let current_row = i / 2;
        let mut pos = Vector2::new(start.x - i % 2 - current_row, start.y + current_row);
        let mut vertical_tiles = columns;

        if pos.x < 0 || pos.y < 0 {
            if pos.x < pos.y {
                vertical_tiles += pos.x;
                pos.y -= pos.x;
                pos.x = 0;
            } else {
                vertical_tiles += pos.y;
                pos.x -= pos.y;
                pos.y = 0;
            }
        }

        pos.x += vertical_tiles;
        pos.y += vertical_tiles;

        if pos.x >= map_size_x {
            let tiles_overflow = pos.x - map_size_x;
            vertical_tiles -= tiles_overflow;
            pos.y -= tiles_overflow;
        }

        if pos.y >= map_size_y {
            let tiles_overflow = pos.y - map_size_y;
            vertical_tiles -= tiles_overflow;
        }

        if vertical_tiles < 0 {
            break;
        }
        visible_index += vertical_tiles;
    }

    visible_index
}
