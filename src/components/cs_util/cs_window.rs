use std::ops::DerefMut;
use log::{error, warn};
use bevy_ecs::prelude::*;
use bevy_ecs::system::adapter::new;
use cgmath::Vector2;
use wgpu::{
    Buffer, Device, PresentMode, Queue, RenderPipeline, Surface, SurfaceConfiguration,
    TextureFormat, TextureUsages,
};
use winit::{
    event::*,
    event_loop::ControlFlow,
    window::Window,
};
use winit::dpi::{LogicalSize, PhysicalSize};

use crate::components::cs_render::shader_types::texture::Texture;
use crate::components::cs_util::camera::CustomCamera;
use crate::components::cs_util::input::Input;
use crate::main_loop::{DummyTest, Render};

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::prelude::*;
        use web_sys;
        use crossbeam_channel::Receiver;
        use web_sys::HtmlElement;

        #[derive(Debug, Clone, Resource)]
        pub struct WinitWebResizing {
            pub rx: Receiver<LogicalSize<f32>>,
        }
    }
}

pub struct State {
    pub(crate) window: Window,
}

impl State {
    async fn new(window: Window) -> (Self, Render) {
        //window
        let render = Self::create_window(&window).await;
        //window end

        (Self {
            window
        }, render)
    }

    async fn create_window(
        window: &Window
    ) -> Render {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");
        Render {
            surface,
            device,
            queue,
            size,
            config,
            depth_texture,
        }
    }

    pub(crate) fn window(&self) -> &Window {
        &self.window
    }

    pub(crate) fn resize(&mut self, world: &mut World) {
        let renderer = world.get_resource::<Render>().unwrap();
        if renderer.size.width <= 0 || renderer.size.height <= 0 {
            return;
        }

        resize_window(world);
    }

    fn input(&mut self, event: &WindowEvent, world: &mut World) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let mut key_input = world.get_resource_mut::<Input<VirtualKeyCode>>().unwrap();
                key_input.bypass_change_detection();

                match state {
                    ElementState::Pressed => key_input.press(*keycode),
                    ElementState::Released => key_input.release(*keycode),
                }
            }
            _ => {}
        };

        false
    }
}

pub fn resize_window(world: &mut World) {
    let mut render = world.get_resource_mut::<Render>().unwrap();
    render.config.width = render.size.width;
    render.config.height = render.size.height;
    render.surface.configure(&render.device, &render.config);
    render.depth_texture = Texture::create_depth_texture(&render.device, &render.config, "depth_texture");


    let size = Vector2::new(render.config.width, render.config.height);
    let mut camera = world.get_resource_mut::<CustomCamera>().unwrap();
    camera.update_projection(Vector2::new(
        size.x as f32,
        size.y as f32,
    ));
}

pub async fn platform_specific_init(window: Window, world: &mut World) -> (State, Render) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");

            let get_full_size = || {
                let win = web_sys::window().unwrap();
                let document = win.document().unwrap();
                let element = document.get_element_by_id("wasm-example").unwrap();
                let element = element.dyn_into::<HtmlElement>().unwrap();
                LogicalSize::new(
                    element.client_width() as f32,
                    element.client_height() as f32,
                )
            };

            let (tx, rx) = crossbeam_channel::unbounded();
            let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {

                tx.send(get_full_size()).unwrap();
            }) as Box<dyn FnMut(_)>);
            let window_sys = web_sys::window().unwrap();
            window_sys
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();

            let (state,render) = State::new(window).await;

            world.insert_resource(WinitWebResizing{rx});
        }else{
            let (state,render) = State::new(window).await;

        }
    }
    (state, render)
}

pub fn check_window_events(state: &mut State, control_flow: &mut ControlFlow, event: &WindowEvent, world: &mut World) {
    if state.input(event, world) { return; }
    match event {
        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
        WindowEvent::Resized(physical_size) => resize(state, world, *physical_size),
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => resize(state, world, **new_inner_size),
        _ => {}
    }
}

fn resize(state: &mut State, world: &mut World, new_inner_size: PhysicalSize<u32>) {
    let mut render = world.get_resource_mut::<Render>().unwrap();
    render.size = new_inner_size;
    state.resize(world);
}
