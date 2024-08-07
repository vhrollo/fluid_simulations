use std::time::Instant;
use log::info;
use crate::state::State;
use crate::state::render::Render;
use cgmath::{Matrix, SquareMatrix, Vector4};
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
        
        println!("{:?}", proj_view_inv * Vector4{x:0.0, y:0.0, z:0.0, w:1.0});


        let proj_view_inv = MatrixUniform { matrix: proj_view_inv.into() };
        self.proj_view_inv = proj_view_inv;
        let debug: cgmath::Matrix4<f32> = self.proj_view_inv.matrix.into();
        
        self.queue.write_buffer(&self.proj_view_inv_buffer, 0, bytemuck::cast_slice(&[proj_view_inv]));
        
        self.queue.write_buffer(&self.view_buffer, 0, bytemuck::cast_slice(&[self.view.view_matrix]));
        self.queue.write_buffer(&self.proj_buffer, 0, bytemuck::cast_slice(&[self.proj.camera_matrix]));

        self.queue.write_buffer(&self.smoothing_pipeline.smoothing_buffer, 0, bytemuck::cast_slice(&[self.water_simulation.smoothing_radius]));
        self.queue.write_buffer(&self.density_pipeline.smoothing_buffer, 0, bytemuck::cast_slice(&[self.water_simulation.smoothing_radius]));
        
        self.queue.write_buffer(&self.water_simulation.num_particles_buffer, 0, bytemuck::cast_slice(&[self.water_simulation.num_particles as u32]));
        self.queue.write_buffer(&self.delta_time_buffer, 0, bytemuck::cast_slice(&[delta_time.as_secs_f32()]));        
        
        self.queue.write_buffer(&self.pressed_buffer, 0, bytemuck::cast_slice(&[self.camera_controller.is_mouse_pressed as u32]));
        self.queue.write_buffer(&self.mouse_delta_buffer, 0, bytemuck::cast_slice(&[self.camera_controller.mouse_delta]));
        
        if !self.paused {
            println!("{:?}", debug * Vector4{x:1.0, y:1.0, z:0.0, w:0.0});
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