use std::iter;


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
        
        {
            let mut compute_pass = encoder.begin_compute_pass(
                &wgpu::ComputePassDescriptor { 
                    label: Some("Compute Pass"), timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.predict_position_pipeline);
            compute_pass.set_bind_group(0, &self.particle_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.settings_bind_group, &[]);
            compute_pass.dispatch_workgroups((self.water_simulation.num_particles + 15)/ 16, 1, 1);
        }

        {
            let mut compute_pass = encoder.begin_compute_pass(
                &wgpu::ComputePassDescriptor { 
                    label: Some("Compute Pass"), timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.calculate_density_pipeline);
            compute_pass.set_bind_group(0, &self.particle_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.settings_bind_group, &[]);
            compute_pass.dispatch_workgroups((self.water_simulation.num_particles + 15)/ 16, 1, 1);
        }
        
        {
            let mut compute_pass = encoder.begin_compute_pass(
                &wgpu::ComputePassDescriptor { 
                    label: Some("Compute Pass"), timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.update_position_pipeline);
            compute_pass.set_bind_group(0, &self.particle_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.settings_bind_group, &[]);
            compute_pass.dispatch_workgroups((self.water_simulation.num_particles + 15)/ 16, 1, 1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        Ok(())
    }
}