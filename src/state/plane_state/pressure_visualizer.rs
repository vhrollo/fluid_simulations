use wgpu::util::DeviceExt;
use wgpu::{Buffer, BufferUsages, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindGroupEntry, PipelineLayoutDescriptor, ShaderModule, ShaderStages, PrimitiveTopology};
use crate::simulation::bounding_box;
use crate::simulation::simulation::WaterSimulation;
use crate::state::managers::pipeline_manager::PipelineManager;
use crate::simulation::bounding_box::BoundingBox;


pub struct PressureVisualizer {
    pub pressure_pipeline: wgpu::RenderPipeline,
}

impl PressureVisualizer {
    pub fn new(
        device: &wgpu::Device, 
        pipeline_manager: &PipelineManager, 
        camera_bind_group_layout: &BindGroupLayout,
        particle_bind_group_layout: &BindGroupLayout, 
        water_simulation: &WaterSimulation, 
        settings_bind_group_layout: &BindGroupLayout,
        ) -> Self {
        
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Pressure Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shader/pressure/pressure.wgsl").into()),
        };


        let pressure_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Smoothing Pipeline Layout"),
            bind_group_layouts: &[
                camera_bind_group_layout,
                particle_bind_group_layout,
                settings_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pressure_pipeline = pipeline_manager.create_wgsl_pipeline(
            &pressure_pipeline_layout,
            None,
            &[],
            PrimitiveTopology::TriangleStrip,
            Some(wgpu::BlendState::ALPHA_BLENDING),
            shader,
        );

        Self {
            pressure_pipeline,
        }
    }
}

