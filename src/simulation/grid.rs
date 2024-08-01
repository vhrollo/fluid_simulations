use bytemuck::{Pod, Zeroable, cast_slice};
use futures::channel::oneshot;
use image::buffer;

pub struct Grid {
    pub spatial_lookup_buffer: wgpu::Buffer,
    pub start_indices_buffer: wgpu::Buffer,
    pub entries_buffer: wgpu::Buffer,
    pub grid_bind_layout: wgpu::BindGroupLayout,
    pub grid_bind_group: wgpu::BindGroup,
} 

impl Grid {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, max_particles: usize) -> Self {
        let spatial_lookup_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Spatial Lookup Buffer"),
            size: (max_particles * std::mem::size_of::<HashCell>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let data = vec![HashCell { particle_index: max_particles as u32, cell_index: max_particles as u32 }; max_particles];

        queue.write_buffer(&spatial_lookup_buffer, 0, bytemuck::cast_slice(&data));

        let start_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Start Indices Buffer"),
            size: (max_particles * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }); 

        let data = vec![max_particles as u32; max_particles];

        queue.write_buffer(&start_indices_buffer, 0, bytemuck::cast_slice(&data));

        let entries_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entries Buffer"),
            size: (max_particles * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let grid_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Grid Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false, 
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false, 
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let grid_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Bind Group"),
            layout: &grid_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: spatial_lookup_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: start_indices_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            spatial_lookup_buffer,
            start_indices_buffer,
            entries_buffer,
            grid_bind_layout,
            grid_bind_group,
        }
    }

    pub async fn print_buffer(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // Create a temporary buffer for reading
        let temp_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Temp Buffer"),
            size: self.spatial_lookup_buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create command encoder for copying data
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Buffer Copy Encoder"),
        });

        // Copy data from the storage buffer to the temporary buffer
        encoder.copy_buffer_to_buffer(
            &self.spatial_lookup_buffer,
            0,
            &temp_buffer,
            0,
            self.spatial_lookup_buffer.size(),
        );

        // Submit the command
        queue.submit(Some(encoder.finish()));

        // Map the temporary buffer
        let buffer_slice = temp_buffer.slice(..);
        let (sender, receiver) = oneshot::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        device.poll(wgpu::Maintain::Wait);

        match receiver.await {
            Ok(Ok(())) => {
                let data = buffer_slice.get_mapped_range();
                let cells = cast_slice::<u8, HashCell>(&data);
                println!("Spatial Lookup Buffer:");
                for cell in cells {
                    println!("{:?}", cell);
                }
                drop(data);
                temp_buffer.unmap();
            }
            _ => {
                println!("Failed to map buffer");
            }
        }
    }

    pub async fn print_buffer2(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // Create a temporary buffer for reading
        let temp_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Temp Buffer"),
            size: self.start_indices_buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create command encoder for copying data
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Buffer Copy Encoder"),
        });

        // Copy data from the storage buffer to the temporary buffer
        encoder.copy_buffer_to_buffer(
            &self.start_indices_buffer,
            0,
            &temp_buffer,
            0,
            self.start_indices_buffer.size(),
        );

        // Submit the command
        queue.submit(Some(encoder.finish()));

        // Map the temporary buffer
        let buffer_slice = temp_buffer.slice(..);
        let (sender, receiver) = oneshot::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        device.poll(wgpu::Maintain::Wait);

        match receiver.await {
            Ok(Ok(())) => {
                let data = buffer_slice.get_mapped_range();
                let cells = cast_slice::<u8, i32>(&data);
                println!("inex Buffer:");
                for cell in cells {
                    println!("{:?}", cell);
                }
                drop(data);
                temp_buffer.unmap();
            }
            _ => {
                println!("Failed to map buffer");
            }
        }
    }
} 

// Hash table element struct
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct HashCell {
    pub particle_index: u32,
    pub cell_index: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Constants {
    pub k: u32,
    pub j: u32,
    pub pwer_of_two: u32,
}
