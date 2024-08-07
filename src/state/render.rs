use std::iter;

use crate::simulation::grid::{Constants, HashCell};
use crate::state::State;

pub trait Render {
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
    fn compute(&mut self) -> Result<(), wgpu::SurfaceError>;
}


impl<'a> Render for State<'a> {
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });



            //first pipeline - pic
            render_pass.set_pipeline(&self.render_pipeline);
            if self.space {
                render_pass.set_bind_group(0, &self.bind_groups[0], &[]);
            } else {
                render_pass.set_bind_group(0, &self.bind_groups[1], &[]);
            }
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices,0, 0..1); // 3.

            //second pipeline - density visualizer
            // render_pass.set_pipeline(&self.density_pipeline.density_vis_pipeline);
            // render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            // render_pass.set_bind_group(1, &self.density_pipeline.smoothing_bind_group, &[]);
            // render_pass.set_bind_group(2, &self.particle_bind_group, &[]);
            // render_pass.draw(0..4, 0..self.water_simulation.num_particles);
            
            //pressure visualizer 
            // render_pass.set_pipeline(&self.pressure_visualizer.pressure_pipeline);
            // render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            // render_pass.set_bind_group(1, &self.particle_bind_group, &[]);
            // render_pass.set_bind_group(2, &self.settings_bind_group, &[]);
            // render_pass.draw(0..4,  0..1);
            
            //third pipeline - particle pipeline
            render_pass.set_pipeline(&self.particle_pipeline);
            render_pass.set_bind_group(0, &self.particle_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(2, &self.radius_bind_group, &[]);
            render_pass.draw(0..4, 0..self.water_simulation.num_particles);

            //fourth pipeline - bounding box
            render_pass.set_pipeline(&self.bounding_box.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.bounding_box.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.bounding_box.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.bounding_box.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.bounding_box.num_indices,0, 0..1); // 3.

            //fifth pipeline - smoothing pipeline
            render_pass.set_pipeline(&self.smoothing_pipeline.smoothing_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.smoothing_pipeline.smoothing_bind_group, &[]);
            render_pass.draw(0..4, 0..1);
            
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        self.fps_tracker.update();
        self.console_logger.fps(self.fps_tracker.get_fps());

        Ok(())
    }

    fn compute(&mut self) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder")
            });
        
        let mut compute_pass = encoder.begin_compute_pass(
            &wgpu::ComputePassDescriptor { 
                label: Some("Compute Pass"), timestamp_writes: None,
        });
        // particle predictioning
        compute_pass.set_pipeline(&self.predict_position_pipeline);
        compute_pass.set_bind_group(0, &self.particle_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.settings_bind_group, &[]);
        compute_pass.set_bind_group(2, &self.grid.grid_bind_group, &[]);
        compute_pass.set_bind_group(3, &self.camera_bind_group_inverse, &[]);
        compute_pass.dispatch_workgroups((self.water_simulation.num_particles + 15)/ 16, 1, 1);
    
        // particle hashing for faster neighbor search
        // let clean_data = vec![HashCell{particle_index: -1, cell_index: -1}; self.water_simulation.max_particles];


        // self.queue.write_buffer(&self.grid.spatial_lookup_buffer, 0, bytemuck::cast_slice(&clean_data));

        compute_pass.set_pipeline(&self.update_spatial_hash_pipeline);
        compute_pass.set_bind_group(0, &self.particle_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.settings_bind_group, &[]);
        compute_pass.set_bind_group(2, &self.grid.grid_bind_group, &[]);
        compute_pass.dispatch_workgroups((self.water_simulation.num_particles + 15)/ 16, 1, 1);
    
        // particle sorting
        let next_power_of_two = 2u32.pow((self.water_simulation.num_particles as f32).log2().ceil() as u32);
        if next_power_of_two > self.water_simulation.max_particles as u32 {
            panic!("Number of particles exceeds maximum number of particles");
        }      
        
        // let buffer_padding_length = next_power_of_two - self.water_simulation.num_particles;
        // let padding = vec![0; buffer_padding_length as usize];
        // println!("num_particles: {}", self.water_simulation.num_particles);
        // println!("next_power_of_two: {}", next_power_of_two);


        // futures::executor::block_on(self.grid.print_buffer(&self.device, &self.queue));

        let mut k = 2u32;
        while k <= next_power_of_two as u32 {
            let mut j = k / 2 as u32;
            while j > 0 {
                compute_pass.set_pipeline(&self.sort_pipeline);
                compute_pass.set_bind_group(0, &self.particle_bind_group, &[]);
                compute_pass.set_bind_group(1, &self.settings_bind_group, &[]);
                compute_pass.set_bind_group(2, &self.grid.grid_bind_group, &[]);
                let constants = Constants { k, j, pwer_of_two: next_power_of_two };
                let constants_data = bytemuck::bytes_of(&constants);
                compute_pass.set_push_constants(0, constants_data);
                compute_pass.dispatch_workgroups((next_power_of_two + 15)/ 16, 1, 1);
                j /= 2;
            }
            k *= 2;
        }

        compute_pass.set_pipeline(&self.reset_indecies_pipeline);
        compute_pass.set_bind_group(0, &self.particle_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.settings_bind_group, &[]);
        compute_pass.set_bind_group(2, &self.grid.grid_bind_group, &[]);
        compute_pass.dispatch_workgroups((self.water_simulation.max_particles as u32 + 15)/ 16, 1, 1);

        // futures::executor::block_on(self.grid.print_buffer(&self.device, &self.queue));
        // calculate_start_indices
        compute_pass.set_pipeline(&self.indecies_pipeline);
        compute_pass.set_bind_group(0, &self.particle_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.settings_bind_group, &[]);
        compute_pass.set_bind_group(2, &self.grid.grid_bind_group, &[]);
        compute_pass.dispatch_workgroups((self.water_simulation.num_particles + 15)/ 16, 1, 1);
        
        // futures::executor::block_on(self.grid.print_buffer2(&self.device, &self.queue));
        // particle density calculation
        compute_pass.set_pipeline(&self.calculate_density_pipeline);
        compute_pass.set_bind_group(0, &self.particle_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.settings_bind_group, &[]);
        compute_pass.set_bind_group(2, &self.grid.grid_bind_group, &[]);
        compute_pass.dispatch_workgroups((self.water_simulation.num_particles + 15)/ 16, 1, 1);
    
        //viscosity calculation
        compute_pass.set_pipeline(&self.viscosity_pipeline);
        compute_pass.set_bind_group(0, &self.particle_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.settings_bind_group, &[]);
        compute_pass.set_bind_group(2, &self.grid.grid_bind_group, &[]);
        compute_pass.dispatch_workgroups((self.water_simulation.num_particles + 15)/ 16, 1, 1);
        
        // particle force calculation
        compute_pass.set_pipeline(&self.update_position_pipeline);
        compute_pass.set_bind_group(0, &self.particle_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.settings_bind_group, &[]);
        compute_pass.set_bind_group(2, &self.grid.grid_bind_group, &[]);
        compute_pass.dispatch_workgroups((self.water_simulation.num_particles + 15)/ 16, 1, 1);
    
        drop(compute_pass);
    
        self.queue.submit(iter::once(encoder.finish()));
        Ok(())
    }
}