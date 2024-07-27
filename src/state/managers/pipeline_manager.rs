use std::primitive;


pub struct PipelineManager<'a> {
    device: &'a wgpu::Device,
    config: &'a wgpu::SurfaceConfiguration,
}

impl<'a> PipelineManager<'a> {
    pub fn new(device: &'a wgpu::Device, config: &'a wgpu::SurfaceConfiguration) -> Self {
        Self { device, config }
    }

    pub fn create_render_pipeline(
        &self,
        label: &str,
        layout: &wgpu::PipelineLayout,
        vertex_shader: &wgpu::ShaderModule,
        fragment_shader: &wgpu::ShaderModule,
        vertex_buffers: &[wgpu::VertexBufferLayout],
        blend: Option<wgpu::BlendState>,
        primitive_topology: wgpu::PrimitiveTopology,
        cull_mode: Option<wgpu::Face>,
    ) -> wgpu::RenderPipeline {
        self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: vertex_shader,
                entry_point: "main",
                compilation_options: Default::default(),
                buffers: vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: fragment_shader,
                entry_point: "main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.config.format,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: primitive_topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }
    

    pub fn create_compute_pipeline(
        &self,
        label: &str,
        layout: &wgpu::PipelineLayout,
        compute_shader: &wgpu::ShaderModule,
        entery_point: &str,
    ) -> wgpu::ComputePipeline {
        self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor   {
            label: Some(label),
            layout: Some(layout),
            module: compute_shader,
            entry_point: entery_point,
            compilation_options: Default::default(),
        })
    }

    pub fn create_wgsl_pipeline(
        &self,
        layout: &wgpu::PipelineLayout,
        depth_format: Option<wgpu::TextureFormat>,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        primitive_topology: wgpu::PrimitiveTopology,
        blend: Option<wgpu::BlendState>,
        shader: wgpu::ShaderModuleDescriptor,
    ) -> wgpu::RenderPipeline {

        let shader = self.device.create_shader_module(shader);

        self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.config.format,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: primitive_topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
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
}