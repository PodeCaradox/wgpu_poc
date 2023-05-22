

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GeometryData {
    pub position: [u8; 4],
    pub tex_coord: [u8; 4],
}

pub const VERTICES: &[GeometryData] = &[

    GeometryData { position: [0, 0, 0, 0], tex_coord: [0, 255, 0, 0] }, // A
    GeometryData { position: [255, 0, 0, 0], tex_coord: [255, 255, 0, 0] }, // B
    GeometryData { position: [0, 255, 0, 0], tex_coord: [0, 0, 0, 0] }, // E

    GeometryData { position: [0, 255, 0, 0], tex_coord: [0, 0, 0, 0] }, // A
    GeometryData { position: [255, 0, 0, 0], tex_coord: [255, 255, 0, 0] }, // B
    GeometryData { position: [255, 255, 0, 0], tex_coord: [255, 0, 0, 0] }, // E


];

impl GeometryData {
    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<GeometryData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Unorm8x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[u8; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Unorm8x4, // NEW!
                },
            ]
        }
    }
}


