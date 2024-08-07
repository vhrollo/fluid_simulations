use std::time::Instant;
use log::info;
use crate::state::State;
use crate::state::render::Render;
use cgmath::{Matrix, SquareMatrix, Vector4, Vector2};
use crate::state::camera::camera::{inverse, CameraMatrix, ViewMatrix, MatrixUniform};


pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);


pub trait OtherLogic {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    fn should_update(&self) -> bool;
    fn timestamp(&mut self);
    fn update(&mut self);
    fn print_adapters(wgpu_instance: &wgpu::Instance);
    fn update_particle_vertex_data(&mut self);
}

impl<'a> OtherLogic for State<'a>{
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
        self.size = new_size;
    }  


    fn should_update(&self) -> bool {
        self.last_update.elapsed() >= self.frame_duration
    }

    fn timestamp(&mut self) {
        self.last_update = Instant::now();
    }

    fn update(&mut self) {
        self.surface_configured = true;
        let delta_time = self.last_update.elapsed();

        // self.camera_controller.yaw_pitch(delta_time);
        self.camera_controller.update_camera(&mut self.view, delta_time);
        self.view.update_view();
        
        self.proj.update_proj(self.size);


        
        let matrix4_proj: cgmath::Matrix4<f32> = self.proj.camera_matrix.matrix.into();
        let matrix4_view: cgmath::Matrix4<f32> = self.view.view_matrix.matrix.into();
        let proj_view_inv = (OPENGL_TO_WGPU_MATRIX * matrix4_proj * matrix4_view).invert().unwrap();
        
        let transformed_point = OPENGL_TO_WGPU_MATRIX * matrix4_proj * matrix4_view * Vector4::new(15.0, 10.0, 0.0, 1.0);
        let ndc = transformed_point / transformed_point.w;


        let mouse_pos_ndc = cgmath::Vector4 {
            x: self.camera_controller.mouse_delta.current_position.x,
            y: self.camera_controller.mouse_delta.current_position.y,
            z: ndc.z,
            w: 1.0
        };
        
        let mouse_pos_world = proj_view_inv * mouse_pos_ndc;
        let world_pos = (cgmath::Vector2 { x: mouse_pos_world.x, y: mouse_pos_world.y } / mouse_pos_world.w);
        // println!("Mouse World Pos: {:?}", world_pos);

        // println!("{:?}", OPENGL_TO_WGPU_MATRIX * matrix4_proj * matrix4_view * Vector4{x:8.5, y:8.5, z:0.0, w:1.0});


        // println!("NDC Coordinates: {:?}", ndc);


        let proj_view_inv = MatrixUniform { matrix: proj_view_inv.into() };
        self.proj_view_inv = proj_view_inv;
        
        self.queue.write_buffer(&self.proj_view_inv_buffer, 0, bytemuck::cast_slice(&[proj_view_inv]));
        self.queue.write_buffer(&self.cheat_depth_buffer, 0, bytemuck::cast_slice(&[ndc.z]));        

        self.queue.write_buffer(&self.view_buffer, 0, bytemuck::cast_slice(&[self.view.view_matrix]));
        self.queue.write_buffer(&self.proj_buffer, 0, bytemuck::cast_slice(&[self.proj.camera_matrix]));

        self.queue.write_buffer(&self.smoothing_pipeline.smoothing_buffer, 0, bytemuck::cast_slice(&[self.water_simulation.smoothing_radius]));
        self.queue.write_buffer(&self.density_pipeline.smoothing_buffer, 0, bytemuck::cast_slice(&[self.water_simulation.smoothing_radius]));
        
        self.queue.write_buffer(&self.water_simulation.num_particles_buffer, 0, bytemuck::cast_slice(&[self.water_simulation.num_particles as u32]));
        self.queue.write_buffer(&self.delta_time_buffer, 0, bytemuck::cast_slice(&[delta_time.as_secs_f32()]));        
        
        self.queue.write_buffer(&self.pressed_buffer, 0, bytemuck::cast_slice(&[self.camera_controller.is_mouse_pressed as u32]));
        self.queue.write_buffer(&self.mouse_delta_buffer, 0, bytemuck::cast_slice(&[self.camera_controller.mouse_delta]));
        
        if !self.paused {
            // self.water_simulation.update(delta_time);
            //self.water_simulation.update_in_parrallel(delta_time);
            // self.updateeparticle_vertex_data();
            match self.compute() {
                Ok(_) => {}
                Err(e) => {
                    info!("Error in compute: {:?}", e);
                }
            }
        }
    }

    fn update_particle_vertex_data(&mut self) {
        self.queue.write_buffer(&self.particle_buffer, 0, bytemuck::cast_slice(&self.water_simulation.particles));
    }

    fn print_adapters(wgpu_instance: &wgpu::Instance) {
        info!("");
        let adapter_info = wgpu_instance.enumerate_adapters(wgpu::Backends::all());
        for adapter_enum in adapter_info {
            info!("{:?}", adapter_enum);
        }
        info!("");
    }
}