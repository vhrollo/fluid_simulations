pub fn create_shader_module(device: &wgpu::Device, label: &str, source: &str, stage: naga::ShaderStage) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Glsl {
            shader: source.into(),
            stage,
            defines: Default::default(),
        },
    })
}

pub fn create_shader_module2(device: &wgpu::Device, label: &str, source: &str, stage: naga::ShaderStage) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}