#[allow(unused_imports)]
use wgpu::util::DeviceExt;


//for vertices with color

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
    velocity: [f32; 3],
}

impl Vertex{
    const ATTRIBS: [wgpu::VertexAttribute; 3] = 
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241 ,  0.49240386, 0.0], color: [0.5, 0.0, 0.0], velocity: [0.0, 0.0, 0.0] },
    Vertex { position: [-0.49513406,  0.06958647, 0.0], color: [0.0, 0.5, 0.0], velocity: [0.0, 0.0, 0.0] },
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.0, 0.0, 1.0], velocity: [0.0, 0.0, 0.0] },
    Vertex { position: [ 0.35966998, -0.3473291 , 0.0], color: [0.0, 0.5, 0.0], velocity: [0.0, 0.0, 0.0] },
    Vertex { position: [ 0.44147372,  0.2347359 , 0.0], color: [0.5, 0.0, 0.0], velocity: [0.0, 0.0, 0.0] },
    Vertex { position: [ 1.0       , -1.0       , 0.0], color: [1.0, 0.0, 0.0], velocity: [0.0, 0.0, 0.0] },
];

pub const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];


//for textures

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexImg {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl VertexImg{
    const ATTRIBS: [wgpu::VertexAttribute; 2] = 
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<VertexImg>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub const VERTICESIMG: &[VertexImg] = &[
    VertexImg { position: [-0.0868241 ,  0.49240386, 0.0], tex_coords: [0.4131759   , 1.0 - 0.99240386], }, // A
    VertexImg { position: [-0.49513406,  0.06958647, 0.0], tex_coords: [0.0048659444, 1.0 - 0.56958647], }, // B
    VertexImg { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453  , 1.0 - 0.05060294], }, // C
    VertexImg { position: [ 0.35966998, -0.3473291 , 0.0], tex_coords: [0.85967     , 1.0 - 0.1526709 ], }, // D
    VertexImg { position: [ 0.44147372,  0.2347359 , 0.0], tex_coords: [0.9414737   , 1.0 - 0.7347359 ], }, // E
];

#[derive(Debug, Clone)]
pub struct SVec<T> {
    pub elements: Vec<T>,
}

impl std::ops::Mul<f32> for &SVec<f32> {
    type Output = SVec<f32>;

    fn mul(self, scalar: f32) -> SVec<f32> {
        SVec {
            elements: self.elements.iter().map(|&x| x * scalar).collect(),
        }
    }
}

impl std::ops::AddAssign<&SVec<f32>> for SVec<f32> {
    fn add_assign(&mut self, other: &SVec<f32>) {
        assert_eq!(self.elements.len(), other.elements.len(), "Vectors must be of the same size");
        self.elements.iter_mut().zip(&other.elements).for_each(|(a, b)| *a += b);
    }
}

impl std::ops::SubAssign<&SVec<f32>> for SVec<f32> {
    fn sub_assign(&mut self, other: &SVec<f32>) {
        assert_eq!(self.elements.len(), other.elements.len(), "Vectors must be of the same size");
        self.elements.iter_mut().zip(&other.elements).for_each(|(a, b)| *a -= b);
    }
}