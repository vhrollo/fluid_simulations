use wgpu::util::DeviceExt;
use std::time::{Duration, Instant};
use crate::utils::{console_logger::ConsoleLogger, fps::FpsTracker};
use crate::simulation::{bounding_box::BoundingBox, simulation::{WaterSimulation, ParticleLl, PositionLl, VelocityLl, DensityLl}};
use super::camera::camera::{ViewMatrix, CameraMatrix};
use super::events::{ApplicationEvent, Update, EventHandler};
use super::plane_state::pressure_visualizer;
use super::update::OtherLogic;
use super::texture::Texture;
use super::camera::camera_controller::CameraController;
use super::managers::{texture_manager::TextureManager, pipeline_manager::PipelineManager};
use super::shader_helper;
use fluid_simulations::{VERTICESIMG, INDICES, VertexImg};
use super::plane_state::{density_visualizer::DensityVisualizer, smoothing_ring::SmoothingPipeline};
use crate::simulation::grid::{Grid, Constants};
use crate::state::camera::camera_controller::MouseDelta;
use crate::state::camera::camera::MatrixUniform;
use crate::state::camera::camera::inverse;
use cgmath::SquareMatrix;

use winit::{
    event::WindowEvent,
    event_loop::EventLoop,
    window::Window,
};



const TARGET_FPS: u32 = 2;

pub struct State<'a> {
    pub window: &'a Window,
    pub surface: wgpu::Surface<'a>,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,


    pub diffuse_texture: Texture,
    pub bind_groups: Vec<wgpu::BindGroup>,
    
    pub surface_configured: bool,
    pub update_mode: Update,
    pub last_update: Instant,
    pub frame_duration: Duration,
    pub space: bool,
    pub paused: bool,

    pub fps_tracker: FpsTracker,
    pub console_logger: ConsoleLogger,

    pub view: ViewMatrix,
    pub proj: CameraMatrix,
    pub view_buffer: wgpu::Buffer,
    pub proj_buffer: wgpu::Buffer,
    pub proj_view_inv_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_bind_group_inverse: wgpu::BindGroup,
    pub proj_view_inv: MatrixUniform,

    pub camera_controller: CameraController,

    pub water_simulation: WaterSimulation,
    pub particle_buffer: wgpu::Buffer,
    pub particle_bind_group: wgpu::BindGroup,
    pub particle_pipeline: wgpu::RenderPipeline,
    pub radius_bind_group: wgpu::BindGroup,

    pub bounding_box: BoundingBox,

    pub smoothing_pipeline: SmoothingPipeline,
    pub density_pipeline: DensityVisualizer,
    pub predict_position_pipeline: wgpu::ComputePipeline,
    pub calculate_density_pipeline: wgpu::ComputePipeline,
    pub update_position_pipeline: wgpu::ComputePipeline,
    pub update_spatial_hash_pipeline: wgpu::ComputePipeline,
    pub sort_pipeline: wgpu::ComputePipeline,
    pub indecies_pipeline: wgpu::ComputePipeline,
    pub reset_indecies_pipeline: wgpu::ComputePipeline,
    pub viscosity_pipeline: wgpu::ComputePipeline,

    pub settings_bind_group: wgpu::BindGroup,
    pub pressure_visualizer: pressure_visualizer::PressureVisualizer,
    pub delta_time_buffer: wgpu::Buffer,
    pub cheat_depth_buffer: wgpu::Buffer,

    pub pressed_buffer: wgpu::Buffer,
    pub mouse_delta_buffer: wgpu::Buffer,

    pub grid: Grid,
}

impl <'a> State <'a> {
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default() 
        };
        let wgpu_instance = wgpu::Instance::new(instance_descriptor);

        let surface = wgpu_instance.create_surface(window).unwrap();
        
        Self::print_adapters(&wgpu_instance);
        

        let adapter = wgpu_instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(), //high preformance later
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        ).await.expect("adapter request");

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("main device"),
                required_features: wgpu::Features::VERTEX_WRITABLE_STORAGE | 
                                    wgpu::Features::PUSH_CONSTANTS,
                required_limits: wgpu::Limits{
                    max_push_constant_size: 12,
                    ..wgpu::Limits::default()
                },
            },
            None,
        ).await.expect("device request");

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = Self::configure_surface(
            size, 
            surface_format, 
            surface_caps.alpha_modes[0]);


        let pipeline_manager = PipelineManager::new(&device, &config);
        
        surface.configure(&device, &config);
        //shorter texture
        let texture_manager = TextureManager::new(&device, &queue);

        // Load textures
        let diffuse_texture = texture_manager.load_texture(include_bytes!("../imgs/p1.png"), "porter");
        let jump_texture = texture_manager.load_texture(include_bytes!("../imgs/p2.jpg"), "anime");

        // Create texture bind group layout and bind groups
        let texture_bind_group_layout = texture_manager.create_texture_bind_group_layout();

        let diffuse_bind_group = texture_manager.create_texture_bind_group(
            &texture_bind_group_layout,
            &diffuse_texture,
            "diffuse_bind_group",
        );

        let jump_bind_group = texture_manager.create_texture_bind_group(
            &texture_bind_group_layout,
            &jump_texture,
            "jump_bind_group",
        );

        let bind_groups = vec![diffuse_bind_group, jump_bind_group];

        let view = ViewMatrix::new(
            [0.0, 0.0, -6.0],
            [0.0, 1.0, 0.0],
            90.0,
            0.0,
        );
        
        let proj = CameraMatrix::new(
            size,
            60.0,
            0.1,
            1024.0
        );

        let matrix4_proj: cgmath::Matrix4<f32> = proj.camera_matrix.matrix.into();
        let matrix4_view: cgmath::Matrix4<f32> = view.view_matrix.matrix.into();
        let proj_view_inv = MatrixUniform { matrix: inverse(matrix4_proj * matrix4_view).into()};


        let camera_controller = CameraController::new(5.0, 20.0, &view);
        let mut water_simulation = WaterSimulation::new(&device);

        let view_buffer = Self::create_init_buffer(
            &device,
             "View Buffer", 
            bytemuck::cast_slice(&[view.view_matrix]), 
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);

        let proj_buffer = Self::create_init_buffer(
            &device,
             "Perspective Buffer", 
            bytemuck::cast_slice(&[proj.camera_matrix]), 
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);

        let proj_view_inv_buffer= Self::create_init_buffer(
            &device,
             "View Buffer Inverse", 
            bytemuck::cast_slice(&[proj_view_inv]), 
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);

        let radius_buffer = Self::create_init_buffer(
            &device,
             "Info Buffer", 
            bytemuck::cast_slice(&[water_simulation.radius]), 
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);

        let vertex_buffer = Self::create_init_buffer(
            &device,
             "Vertex Buffer", 
            bytemuck::cast_slice(VERTICESIMG), 
            wgpu::BufferUsages::VERTEX);

        let index_buffer = Self::create_init_buffer(
            &device,
             "Index Buffer", 
            bytemuck::cast_slice(INDICES), 
            wgpu::BufferUsages::INDEX);
        
        
        let radius_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("radius_bind_group_layout"),
        });

        let radius_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &radius_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: radius_buffer.as_entire_binding(),
            }],
            label: Some("radius_bind_grou"),
        });


        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_inverse_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("camera_bind_group_layout"),
        }); 


        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: view_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: proj_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let camera_bind_group_inverse = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_inverse_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: proj_view_inv_buffer.as_entire_binding(),
                },
            ],
            label: Some("camera_bind_group_inverse"),
        });

        let particale_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | 
                                wgpu::ShaderStages::COMPUTE| 
                                wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("particle_bind_group_layout"),
        });
        
        let particle_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Particle Buffer"),
                size: (water_simulation.max_particles * std::mem::size_of::<ParticleLl>()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        let position_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Position Buffer"),
                size: (water_simulation.max_particles * std::mem::size_of::<PositionLl>()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        
        let velocity_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Velocity Buffer"),
                size: (water_simulation.max_particles * std::mem::size_of::<VelocityLl>()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        let density_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Density Buffer"),
                size: (water_simulation.max_particles * std::mem::size_of::<DensityLl>()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        let predicted_position_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Predicted Position Buffer"),
                size: (water_simulation.max_particles * std::mem::size_of::<PositionLl>()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        let bounding_box = BoundingBox::new(
            cgmath::vec2(0.0, 0.0),
            cgmath::vec2(water_simulation.bound_size[0], water_simulation.bound_size[1]),
            &device,
            &pipeline_manager,
            &camera_bind_group_layout,
        );



        water_simulation.add_multiple_uniform_particles(500, &queue, &particle_buffer, &position_buffer, &velocity_buffer, &density_buffer);


        let particle_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &particale_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: position_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: velocity_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: density_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: predicted_position_buffer.as_entire_binding(),
                },
            ],
            label: Some("particle_bind_group"),
        });

        let vert_texture_shader = shader_helper::create_shader_module(&device, "Vert Texture Shader", include_str!("../shader/texture/texture.vert"), naga::ShaderStage::Vertex);
        let frag_texture_shader = shader_helper::create_shader_module(&device, "Frag Texture Shader", include_str!("../shader/texture/texture.frag"), naga::ShaderStage::Fragment);
        let vert_particle_shader = shader_helper::create_shader_module(&device, "Vert Particle Shader", include_str!("../shader/particle/particle.vert"), naga::ShaderStage::Vertex);
        let frag_particle_shader = shader_helper::create_shader_module(&device, "Frag Particle Shader", include_str!("../shader/particle/particle.frag"), naga::ShaderStage::Fragment);
        let compute_shader = shader_helper::create_shader_module2(&device, "Compute Shader", include_str!("../shader/compute/simulation.wgsl"), naga::ShaderStage::Compute);


        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bind_group_layout,
            ], 
            push_constant_ranges: &[],
        });

        let particle_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Pipeline Layout"),
            bind_group_layouts: &[
                &particale_bind_layout,
                &camera_bind_group_layout,
                &radius_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = pipeline_manager.create_render_pipeline(
            "Render Pipeline",
            &render_pipeline_layout,
            &vert_texture_shader,
            &frag_texture_shader,
            &[VertexImg::desc()],
            Some(wgpu::BlendState::REPLACE),
            wgpu::PrimitiveTopology::TriangleList,
            Some(wgpu::Face::Back),
        );

        let particle_pipeline = pipeline_manager.create_render_pipeline(
            "Particle Pipeline",
            &particle_pipeline_layout,
            &vert_particle_shader,
            &frag_particle_shader,
            &[],
            Some(wgpu::BlendState::REPLACE),
            wgpu::PrimitiveTopology::TriangleStrip,
            None,
        );

        let smoothing_pipeline = SmoothingPipeline::new(
            &device,
            &pipeline_manager,
            &camera_bind_group_layout,
            &water_simulation,
        );
        
        let density_pipeline = DensityVisualizer::new(
            &device,
            &pipeline_manager,
            &camera_bind_group_layout,
            &particale_bind_layout,
            &water_simulation,
        );

        let delta_time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Delta Time Buffer"),
            contents: bytemuck::cast_slice(&[0.0f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let max_particles_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Max Particles Buffer"),
            contents: bytemuck::cast_slice(&[water_simulation.max_particles as u32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pressed_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pressed Buffer"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let mouse_delta_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mouse Delta Buffer"),
            contents: bytemuck::cast_slice(&[MouseDelta{previous_position: cgmath::vec2(0.0, 0.0), current_position: cgmath::vec2(0.0, 0.0)}]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let cheat_depth_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cheat Depth Buffer"),
            contents: bytemuck::cast_slice(&[0.33f32]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });


        let settings_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("settings_bind_layout"),
        });

        let settings_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &settings_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: radius_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: water_simulation.num_particles_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: bounding_box.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: delta_time_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: max_particles_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: pressed_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: mouse_delta_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: cheat_depth_buffer.as_entire_binding(),
                },
            ],
            label: Some("settings_bind_group"),
        });
        
        let grid = Grid::new(&device, &queue, water_simulation.max_particles);

        let compute_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Simulation Pipeline Layout"),
            bind_group_layouts: &[
                &particale_bind_layout,
                &settings_bind_layout,
                &grid.grid_bind_layout,
            ],
            push_constant_ranges: &[],
        });

        let compute_mouse_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Simulation Pipeline Layout"),
            bind_group_layouts: &[
                &particale_bind_layout,
                &settings_bind_layout,
                &grid.grid_bind_layout,
                &camera_bind_inverse_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let gpu_sort_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GPU Sort Pipeline Layout"),
            bind_group_layouts: &[
                &particale_bind_layout,
                &settings_bind_layout,
                &grid.grid_bind_layout,
            ],
            push_constant_ranges: &[
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::COMPUTE,
                    range: 0..12,
                },
            ],
        });

        let predict_position_pipeline = pipeline_manager.create_compute_pipeline(
            "predict_position_compute_pipeline", 
            &compute_mouse_layout, 
            &compute_shader,
            "predict_position",
        );

        let calculate_density_pipeline = pipeline_manager.create_compute_pipeline(
            "calculate_density_compute_pipeline", 
            &compute_layout, 
            &compute_shader,
            "calculate_density",
        );

        let update_position_pipeline = pipeline_manager.create_compute_pipeline(
            "update_position_compute_pipeline", 
            &compute_layout, 
            &compute_shader,
            "update_position",
        );

        let update_spatial_hash_pipeline = pipeline_manager.create_compute_pipeline(
            "spatial_hash_compute_pipeline", 
            &compute_layout, 
            &compute_shader,
            "update_spatial_hash",
        );

        let sort_pipeline = pipeline_manager.create_compute_pipeline(
            "sort_compute_pipeline", 
            &gpu_sort_layout, 
            &compute_shader,
            "bitonic_sort_kernel",
        );

        let indecies_pipeline = pipeline_manager.create_compute_pipeline(
            "indecies_compute_pipeline", 
            &compute_layout, 
            &compute_shader,
            "calculate_start_indices",
        );

        let reset_indecies_pipeline = pipeline_manager.create_compute_pipeline(
            "reset_indecies_compute_pipeline", 
            &compute_layout, 
            &compute_shader,
            "reset_indecies",
        );

        let viscosity_pipeline = pipeline_manager.create_compute_pipeline(
            "viscosity_compute_pipeline", 
            &compute_layout, 
            &compute_shader,
            "calculate_viscosity",
        );

        let pressure_visualizer = pressure_visualizer::PressureVisualizer::new(
            &device,
            &pipeline_manager,
            &camera_bind_group_layout,
            &particale_bind_layout,
            &water_simulation,
            &settings_bind_layout,
        );






        let num_indices = INDICES.len() as u32;

        Self {
            window,
            surface,

            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_texture,
            bind_groups,

            surface_configured: false,
            last_update: Instant::now(),
            frame_duration: Duration::from_secs_f64(1.0 / TARGET_FPS as f64),
            update_mode: Update::Continuous,
            space: false,
            paused: false,

            fps_tracker: FpsTracker::new(),
            console_logger: ConsoleLogger::new(),

            view,
            proj,
            view_buffer,
            proj_buffer,
            proj_view_inv_buffer,
            camera_bind_group,
            camera_bind_group_inverse,
            proj_view_inv,

            camera_controller,

            water_simulation,
            particle_buffer,
            particle_bind_group,
            particle_pipeline,
            radius_bind_group,

            bounding_box,

            smoothing_pipeline,
            density_pipeline,
            predict_position_pipeline,
            calculate_density_pipeline,
            update_position_pipeline,
            update_spatial_hash_pipeline,
            sort_pipeline,
            indecies_pipeline,
            reset_indecies_pipeline,
            viscosity_pipeline,

            settings_bind_group,
            pressure_visualizer,
            delta_time_buffer,
            cheat_depth_buffer,

            pressed_buffer,
            mouse_delta_buffer,

            grid,
        }
    }

    

    fn create_init_buffer(device: &wgpu::Device, label: &str, contents: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents,
            usage,
        })
    }

    fn configure_surface(
        size: winit::dpi::PhysicalSize<u32>, 
        format: wgpu::TextureFormat, 
        alpha_mode: wgpu::CompositeAlphaMode) -> wgpu::SurfaceConfiguration {
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode,
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event, self.size)
    }

    pub fn run(mut self, event_loop: EventLoop<ApplicationEvent>) {
        #[allow(unused)] //men fr jeg b;r nok bruke den
        let proxy_event_loop = event_loop.create_proxy();

        let _ = event_loop.run(move |event, control_flow| {
            self.process_event_loop(event, control_flow);
        });
    } 
}