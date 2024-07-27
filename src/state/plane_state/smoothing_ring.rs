use wgpu::util::DeviceExt;
use wgpu::{Buffer, BufferDescriptor, BufferUsages, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindGroupEntry, PipelineLayout, PipelineLayoutDescriptor, ShaderModule, ShaderStages, PrimitiveTopology};
use crate::simulation::simulation::WaterSimulation;
use crate::state::managers::pipeline_manager::PipelineManager;
use crate::state::shader_helper;


pub struct SmoothingPipeline {
    pub smoothing_buffer: Buffer,
    pub smoothing_bind_group: BindGroup,
    pub smoothing_pipeline: wgpu::RenderPipeline,
}

impl SmoothingPipeline {
    pub fn new(
        device: &wgpu::Device, 
        pipeline_manager: &PipelineManager, 
        camera_bind_group_layout: &BindGroupLayout,
        water_simulation: &WaterSimulation) -> Self {
        let smoothing_buffer = Self::create_smoothing_buffer(device, water_simulation.smoothing_radius);

        let smoothing_vert = shader_helper::create_shader_module(&device, "Smoothing Particle Shader", include_str!("../../shader/smoothing/smoothing.vert"), naga::ShaderStage::Vertex);
        let smoothing_frag = shader_helper::create_shader_module(&device, "Smoothing Particle Shader", include_str!("../../shader/smoothing/smoothing.frag"), naga::ShaderStage::Fragment);
        
        let smoothing_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("smoothing_bind_group_layout"),
        });

        let smoothing_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &smoothing_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: smoothing_buffer.as_entire_binding(),
                }
            ],
            label: Some("smoothing_bind_group"),
        });

        let smoothing_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Smoothing Pipeline Layout"),
            bind_group_layouts: &[
                camera_bind_group_layout,
                &smoothing_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let smoothing_pipeline = pipeline_manager.create_render_pipeline(
            "Smoothing Pipeline",
            &smoothing_pipeline_layout,
            &smoothing_vert,
            &smoothing_frag,
            &[],
            Some(wgpu::BlendState::REPLACE),
            PrimitiveTopology::TriangleStrip,
            None,
        );

        Self {
            smoothing_buffer,
            smoothing_bind_group,
            smoothing_pipeline,
        }
    }

    fn create_smoothing_buffer(device: &wgpu::Device, smoothing_radius: f32) -> Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Smoothing Buffer"),
                contents: bytemuck::cast_slice(&[smoothing_radius]),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            }
        )
    }
}
