use std::path::Path;
use std::time::Duration;

use bevy_ecs::change_detection::DetectChangesMut;
use bevy_ecs::schedule::Schedule;
use bevy_ecs::system::{Resource};
use bevy_ecs::world::World;
use cgmath::Vector2;
use instant::Instant;
use wgpu::{BindGroup, Buffer, ComputePipeline, Device, Queue, RenderPipeline, Surface, SurfaceConfiguration};
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::VirtualKeyCode;
use winit::event_loop::{EventLoop};
use winit::window::{WindowBuilder};

use crate::components::cs_io;
use crate::components::cs_io::{AssetIo, loading_state};
use crate::components::cs_render::render_loop::{render_game_world};
use crate::components::cs_render::shader::{texture_sampler_binding};
use crate::components::cs_render::shader::camera_binding::CameraBinding;
use crate::components::cs_render::shader::compute_shader_binding::ComputeParamsBinding;
use crate::components::cs_render::shader_types::camera_uniform::CameraUniform;
use crate::components::cs_render::shader_types::compute_params_uniform::ComputeParamsUniform;
use crate::components::cs_render::shader_types::texture::Texture;
use crate::components::cs_render::world_render_pipline;
use crate::components::cs_util::camera::CustomCamera;
use crate::components::cs_util::cs_window::{check_window_events, platform_specific_init, State};
#[cfg(target_arch = "wasm32")]
use crate::components::cs_util::cs_window::WinitWebResizing;
use crate::components::cs_util::fps_counter::FPSCounter;
use crate::components::cs_util::input::Input;
use crate::components::cs_world::map;
use crate::components::cs_world::map::SIZE;

#[derive(Resource)]
pub struct Render {
    pub(crate) surface: Surface,
    pub(crate) device: Device,

    pub(crate) queue: Queue,
    pub(crate) size: PhysicalSize<u32>,
    pub(crate) config: SurfaceConfiguration,
    pub(crate) depth_texture: Texture,
}

#[derive(Resource)]
pub struct DummyTest {
    pub(crate) render_pipeline: RenderPipeline,
    pub(crate) compute_pipeline: ComputePipeline,
    pub(crate) geometry_buffer: Buffer,
    pub(crate) diffuse_bind_group: BindGroup,
    pub(crate) instance_buffer_bind_group: BindGroup,
    pub(crate) compute_buffer_bind_group: BindGroup,
    pub(crate) compute_visible_buffer_bind_group: BindGroup,

}

pub async fn main_loop(width: u32, height: u32) {
    let mut world = World::new();

    let (event_loop, mut state, render) = init_window(&mut world, width, height).await;


    let asset_io: Box<dyn AssetIo> = cs_io::get_asset_store();

    //entity world

    let mut update_schedule = Schedule::default();
    //

    //image i draw
    let diffuse_bytes = asset_io.load_path(Path::new("assets\\tiles30x64.png")).await;
    let value_image = diffuse_bytes.unwrap();
    let diffuse_image = Texture::from_bytes(&render.device, &render.queue, &value_image, "diffuse_texture").unwrap();
    let texture_bind_group_layout = texture_sampler_binding::create_texture_group_layout(&render.device);
    let diffuse_bind_group = texture_sampler_binding::create_diffuse_bind_group(
        &render.device,
        &diffuse_image,
        &texture_bind_group_layout,
    );
    //

    //render pipline
    let shader = world_render_pipline::load_shader(
        &render.device,
        &asset_io,
        "assets\\shaders\\instancing.wgsl",
    ).await;
    let geometry_buffer = world_render_pipline::create_geometry_buffer(&render.device);
    //

    //camera
    let camera = CustomCamera::new(
        Vector2::new((SIZE / 2) as f32, (SIZE/2) as f32),
        //Vector2::new((0 / 2) as f32, (0/2) as f32),
        Vector2::new(render.config.width as f32, render.config.height as f32),
        &mut world,
        &mut update_schedule,
    );
    let camera_uniform = CameraUniform::new(&camera, &mut world, &mut update_schedule);
    let camera_bind_group = CameraBinding::new(&render.device, &camera_uniform, &mut world, &mut update_schedule);
    //camera end

    //map stuff
    let mut map = map::generate_instances();
    //map stuff end

    let (instance_buffer_bind_group_layout, instance_buffer_bind_group, visible_tiles_buffer) = world_render_pipline::create_visible_buffer(&render.device, &mut map.tiles);


    let bind_group_layout = [
        &texture_bind_group_layout,
        &camera_bind_group,
        &instance_buffer_bind_group_layout,
    ];

    let render_pipeline = world_render_pipline::create_render_pipline(
        &render.device,
        &render.config,
        &shader,
        &bind_group_layout,
    );

    let compute_params_uniform = ComputeParamsUniform::new(&camera, &mut world, &mut update_schedule);
    let compute_params_bind_group = ComputeParamsBinding::new(&render.device, &compute_params_uniform, &mut world, &mut update_schedule);
    let (compute_buffer_bind_group_layout, compute_buffer_bind_group) = world_render_pipline::create_compute_all_tiles_buffer(&render.device, &mut map.tiles);
    let (compute_visible_buffer_bind_group_layout, compute_visible_buffer_bind_group) = world_render_pipline::create_compute_visible_tiles_buffer(&render.device, visible_tiles_buffer);

    let bind_group_layout = [
        &compute_params_bind_group,
        &compute_buffer_bind_group_layout,
        &compute_visible_buffer_bind_group_layout,
    ];

    let compute_pipeline = world_render_pipline::create_compute_pipline(
        &render.device,
        &shader,
        &bind_group_layout,
    );

    let dummy_test = DummyTest {
        render_pipeline,
        compute_pipeline,
        geometry_buffer,
        diffuse_bind_group,
        instance_buffer_bind_group,
        compute_buffer_bind_group,
        compute_visible_buffer_bind_group,
    };

    world.insert_resource(dummy_test);
    world.insert_resource(<Input<VirtualKeyCode>>::default());
    world.insert_resource(render);
    loading_state::set_loading_finish();
    let mut fps_counter = FPSCounter::new();
    let mut _t: f64 = 0.0;
    let dt: f64 = 0.01;
    let mut current_time = Instant::now();
    let mut accumulator = 0.0;
    event_loop.run(move |event, _, control_flow| {
        match event {
            winit::event::Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                check_window_events(&mut state, control_flow, event, &mut world);
            }

            winit::event::Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                //https://gafferongames.com/post/fix_your_timestep/
                let new_time = Instant::now();
                let mut frame_time = new_time - current_time;
                if frame_time.as_secs_f64() > 0.25 {
                    frame_time = Duration::from_secs_f64(0.25);
                }
                current_time = new_time;

                accumulator += frame_time.as_secs_f64();
                fps_counter.tick(frame_time);
                while accumulator >= dt {
                    _t += dt;

                    update_schedule.run(&mut world);

                    let mut key_input = world.get_resource_mut::<Input<VirtualKeyCode>>().unwrap();
                    key_input.bypass_change_detection();
                    key_input.clear();


                    accumulator -= dt;
                }

                let _alpha = accumulator / dt;

                render_game_world(&mut world, &mut state, control_flow);
            }
            winit::event::Event::MainEventsCleared => {
                cfg_if::cfg_if! {
                    if #[cfg(target_arch = "wasm32")] {
                        let rx = world.get_resource::<WinitWebResizing>().unwrap().clone().rx;
                        if let Some(size) = rx.try_iter().last() {
                            state.window.set_inner_size(size);
                        }
                    }
                }
            }
            winit::event::Event::RedrawEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}

async fn init_window(world: &mut World, width: u32, height: u32) -> (EventLoop<()>, State, Render) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_inner_size(LogicalSize::new(width, height)).build(&event_loop).unwrap();


    //window.set_fullscreen(Some(Fullscreen::Borderless(None)));
    let (state, render) = platform_specific_init(window, world).await;
    (event_loop, state, render)
}