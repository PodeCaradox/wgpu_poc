#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TileInstance {
    pub position: [f32; 3],
    pub atlas_coordinate: AtlasCoordinate,
}


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AtlasCoordinate
{
    pub coordinate: [u8; 2],
    pub index: u16,
}

