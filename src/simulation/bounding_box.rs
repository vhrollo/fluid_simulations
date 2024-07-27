use cgmath::Vector2;
use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, BindGroupLayout};

use crate::state::{managers::pipeline_manager::PipelineManager, shader_helper::create_shader_module};


pub struct BoundingBox {
    position: Vector2<f32>,
    size: Vector2<f32>,
    pub buffer: wgpu::Buffer,
    binding: BoundingBoxLl,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::RenderPipeline,
}

impl BoundingBox {

    pub fn new(position: Vector2<f32>, size: Vector2<f32>, device: &wgpu::Device, pipeline_manager: &PipelineManager, camera_group_layout: &BindGroupLayout) -> Self {

        let binding = Self::make_binding(position, size);
        let buffer = Self::make_buffer(device, binding);
        let vertex_buffer = Self::make_vertex_buffer(device);
        let index_buffer = Self::make_index_buffer(device);
        let num_indices = INDICES.len() as u32;
        let group_layout = Self::make_group_layout(device);
        let bind_group = Self::make_bind_group(device, &group_layout, &buffer);
        let (vert_shader, frag_shader) = Self::make_shader(device);
        let pipeline_layout = Self::make_pipeline_layout(device, &group_layout, camera_group_layout);
        let pipeline = Self::make_pipeline(pipeline_manager, &vert_shader, &frag_shader, &pipeline_layout);

        Self {
            position,
            size,
            buffer,
            binding,
            vertex_buffer,
            index_buffer,
            num_indices,
            bind_group_layout: group_layout,
            bind_group,
            pipeline,
        }
    }

    fn make_buffer(device: &wgpu::Device, binding: BoundingBoxLl) -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Bounding Box Buffer"),
                contents: bytemuck::cast_slice(&[binding]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }
        )
    }

    fn make_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Bounding Box Vertex Buffer"),
                contents: bytemuck::cast_slice(&VERTEX),
                usage: wgpu::BufferUsages::VERTEX,
            }
        )
    }

    fn make_index_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Bounding Box Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        )
    }

    fn make_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("bounding_box_bind_group_layout"),
        })
    }

    fn make_bind_group(
        device: &wgpu::Device, 
        group_layout: &wgpu::BindGroupLayout,
        buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("bounding_box_bind_group"),
        })
    }

    fn make_shader(device: &wgpu::Device) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
        let vert_bounding_box_shader = create_shader_module(
            &device, "Vert Bounding Box Shader", 
            include_str!("../shader/bounding/bounding_box.vert"
            ), 
            naga::ShaderStage::Vertex);
        let frag_bounding_box_shader = create_shader_module(
            &device, "Frag Bounding Box Shader", 
            include_str!("../shader/bounding/bounding_box.frag"
            ), 
            naga::ShaderStage::Fragment);    
        (vert_bounding_box_shader, frag_bounding_box_shader)
    }

    fn make_binding(position: Vector2<f32>, size: Vector2<f32>) -> BoundingBoxLl {
        BoundingBoxLl::new(position, size)
    }

    fn make_pipeline_layout(device: &wgpu::Device, bind_group_layout: &BindGroupLayout, camera_group_layout: &BindGroupLayout) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Bounding Box Pipeline Layout"),
            bind_group_layouts: &[
                camera_group_layout,
                bind_group_layout
                ],
            push_constant_ranges: &[],
        })
    }

    fn make_pipeline(
        pipeline_manager: &PipelineManager,
        vert_shader: &wgpu::ShaderModule,
        frag_shader: &wgpu::ShaderModule,
        pipeline_layout: &wgpu::PipelineLayout) -> wgpu::RenderPipeline {
        pipeline_manager.create_render_pipeline(
            "Bounding Box Pipeline",
            pipeline_layout,
            vert_shader,
            frag_shader,
            &[Vertex::desc()],
            Some(wgpu::BlendState::REPLACE),
            wgpu::PrimitiveTopology::LineList,
            None,
        )
    }

    pub fn update(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.binding]));
    }

}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct BoundingBoxLl {
    position: Vector2<f32>,
    size: Vector2<f32>,
}

unsafe impl Pod for BoundingBoxLl {}
unsafe impl Zeroable for BoundingBoxLl {}

impl BoundingBoxLl {
    fn new(position: Vector2<f32>, size: Vector2<f32>) -> Self {
        Self {
            position,
            size,
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: Vector2<f32>,
}
unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

impl Vertex{
    const ATTRIBS: [wgpu::VertexAttribute; 1] = 
        wgpu::vertex_attr_array![0 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const VERTEX: [Vertex; 4] = [
    Vertex { position: Vector2 { x: -1.0, y: -1.0 } },
    Vertex { position: Vector2 { x: 1.0, y: -1.0 } },
    Vertex { position: Vector2 { x: 1.0, y: 1.0 } },
    Vertex { position: Vector2 { x: -1.0, y: 1.0 } },
];

const INDICES: &[u16] = &[
    0, 1,
    1, 2,
    2, 3,
    3, 0,
];
