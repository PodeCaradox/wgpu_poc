use std::borrow::Cow;
use std::path::Path;

use wgpu::{BindGroup, BindGroupLayout, Buffer, ComputePipeline, Device, RenderPipeline, ShaderModule, SurfaceConfiguration, util};
use wgpu::util::DeviceExt;

use crate::components::cs_io::AssetIo;
use crate::components::cs_render::shader_types::geometry::{GeometryData, VERTICES};
use crate::components::cs_render::shader_types::texture::Texture;
use crate::components::cs_render::shader_types::tile_instance::TileInstance;

pub fn create_render_pipline(device: &Device, config: &SurfaceConfiguration, shader: &ShaderModule, bind_group_layout: &[&BindGroupLayout]) -> RenderPipeline {
    let render_pipeline_layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: bind_group_layout,
            push_constant_ranges: &[],
        }
    );

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[
                GeometryData::desc(),
            ],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Front),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub fn create_visible_buffer(device: &Device, instances: &mut Vec<TileInstance>) -> (BindGroupLayout, BindGroup, Buffer) {

    // let visible_tiles_buffer = device.create_buffer(&BufferDescriptor {
    //     label: Some("visible_tiles_buffer"),
    //     usage: wgpu::BufferUsages::STORAGE,
    //     size: instances.len() as u64,
    //     mapped_at_creation: false,
    // });

    let visible_tiles_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("visible_tiles_buffer"),
        usage: wgpu::BufferUsages::STORAGE,
        contents: bytemuck::cast_slice(&instances),
    });


    let instance_buffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        ],
        label: Some("visible_tiles_bind_group_layout"),
    });

    let instance_buffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &instance_buffer_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: visible_tiles_buffer.as_entire_binding(),
            }
        ],
        label: Some("visible_tiles_bind_group"),
    });
    (instance_buffer_bind_group_layout, instance_buffer_bind_group, visible_tiles_buffer)
}

pub fn create_compute_visible_tiles_buffer(device: &Device, visible_tiles_buffer: Buffer) -> (BindGroupLayout, BindGroup) {
    let instance_buffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some("visible_tiles_bind_group_layout"),
    });

    let instance_buffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &instance_buffer_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: visible_tiles_buffer.as_entire_binding(),
            }
        ],
        label: Some("visible_tiles_bind_group"),
    });
    (instance_buffer_bind_group_layout, instance_buffer_bind_group)
}

pub fn create_compute_all_tiles_buffer(device: &Device, instances: &mut Vec<TileInstance>) -> (BindGroupLayout, BindGroup) {
    let all_tiles_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
        label: Some("all_tiles_buffer"),
        contents: bytemuck::cast_slice(&instances),
        usage: wgpu::BufferUsages::STORAGE,
    }
    );

    let instance_buffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some("all_tiles_bind_group_layout"),
    });

    let instance_buffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &instance_buffer_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: all_tiles_buffer.as_entire_binding(),
            }
        ],
        label: Some("all_tiles_bind_group"),
    });
    (instance_buffer_bind_group_layout, instance_buffer_bind_group)
}

pub fn create_compute_pipline(device: &Device, shader: &ShaderModule, bind_group_layout: &[&BindGroupLayout]) -> ComputePipeline {
    let compute_pipeline_layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: bind_group_layout,
            push_constant_ranges: &[],
        }
    );

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("cs compute pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &shader,
        entry_point: "calcvisibility",
    })
}

pub(crate) async fn load_shader(device: &Device, asset_io: &Box<dyn AssetIo>, path: &str) -> ShaderModule {
    let shader_bytes = asset_io.load_path(Path::new(path)).await;
    let value = shader_bytes.unwrap();
    let shader_string = std::str::from_utf8(&value).unwrap();

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::from(shader_string)),
    });
    shader
}

pub(crate) fn create_geometry_buffer(device: &Device) -> Buffer {
    device.create_buffer_init(
        &util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        }
    )
}