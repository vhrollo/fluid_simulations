use rand::Rng;
use wgpu::core::device;
use std::{num::ParseIntError, time::Duration, vec};
use cgmath::{Vector2, prelude::InnerSpace, Vector3};
use bytemuck::{Pod, Zeroable};
use rayon::prelude::*;

const GRAVITY: f32 = 0.1;
const COLLISION_DAMPING: f32 = 0.8;

pub struct WaterSimulation {
    pub particles: Vec<ParticleLl>,
    pub positions : Vec<PositionLl>,
    pub velocities: Vec<VelocityLl>,
    pub densities: Vec<DensityLl>,
    pub particle_info: Vec<Particle>,
    pub num_particles: u32,
    pub num_particles_buffer: wgpu::Buffer,
    pub max_particles: usize,
    pub bound_size: [f32; 2],
    pub radius: RadiusLl,
    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
}

impl WaterSimulation {
    pub fn new(device: &wgpu::Device) -> Self {
        let num_particles_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("num_particles_buffer"),
            size: std::mem::size_of::<u32>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            particles: Vec::new(),
            particle_info: Vec::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
            densities: Vec::new(),
            num_particles: 0,
            num_particles_buffer,
            max_particles: 2u32.pow(12) as usize,
            bound_size: [30.0, 20.0], //x, y
            radius: RadiusLl::new(0.08),
            smoothing_radius: 0.1,
            target_density: 4.0,
            pressure_multiplier: 0.01,
        }
    }

    // pub fn update(&mut self, delta_time: Duration) {
    //     self.update_density();


    //     for particle_index in 0..self.num_particles {
    //         self.particle_info[particle_index as usize].apply_gravity(delta_time);
    //         let pressure_force = self.calclulate_pressure_force(particle_index, delta_time);
    //         let pressure_acceleration = pressure_force / self.particle_info[particle_index as usize].density;
    //         self.particle_info[particle_index as usize].velocity += pressure_acceleration * 0.001;
    //     }

    //     for particle in self.particle_info.iter_mut() {
    //         particle.position += particle.velocity;
    //     }

    //     for (i, particle) in self.particles.iter_mut().enumerate() {
    //         particle.position = self.particle_info[i].position;
    //         println!("Particle position: {:?}", particle.position);
    //     }

    //     self.collision_detection();

    // }

    // pub fn update_in_parrallel(&mut self, delta_time: Duration) {
    //     self.update_density();

    //     // Calculate pressure forces in parallel
    //     let pressure_forces: Vec<_> = (0..self.num_particles)
    //         .into_par_iter()
    //         .map(|particle_index| {
    //             //self.particle_info[particle_index as usize].apply_gravity(delta_time);
    //             let pressure_force = self.calclulate_pressure_force(particle_index, delta_time);
    //             let pressure_acceleration = pressure_force / self.particle_info[particle_index as usize].density;
    //             pressure_acceleration * 0.01 /*+ Vector3::new(0.0, -GRAVITY * delta_time.as_secs_f32(), 0.0)*/
    //         })
    //         .collect();

    //     // Apply pressure forces in parallel
    //     self.particle_info.par_iter_mut().zip(pressure_forces.par_iter())
    //         .for_each(|(particle, &pressure_acceleration)| {
    //             particle.velocity += pressure_acceleration;
    //         });

    //     // Update positions in parallel
    //     self.particle_info.par_iter_mut().for_each(|particle| {
    //         particle.position += particle.velocity;
    //     });

    //     // Update the positions in ParticleLl based on Particle info
    //     self.particles.par_iter_mut().enumerate().for_each(|(i, particle)| {
    //         particle.position = self.particle_info[i].position;
    //     });

    //     self.collision_detection();
    // }



    // fn collision_detection(&mut self) {
        
    //     let half_bound_sizes = [
    //         (self.bound_size[0] / 2.0) - self.radius.radius, 
    //         (self.bound_size[1] / 2.0) - self.radius.radius,
    //     ];

    //     for particle in self.particle_info.iter_mut() {
    //         if particle.position.x.abs() > half_bound_sizes[0] {
    //             particle.position.x = half_bound_sizes[0] * particle.position.x.signum();
    //             particle.velocity.x *= -1.0;
    //         }

    //         if particle.position.y.abs() > half_bound_sizes[1] {
    //             particle.position.y = half_bound_sizes[1] * particle.position.y.signum();
    //             particle.velocity.y *= -1.0;
    //         }
    //     }

    // }

    pub fn add_multiple_random_particles(&mut self, num_new: u32, queue: &wgpu::Queue, particle_buffer: &wgpu::Buffer, position_buffer: &wgpu::Buffer, velocity_buffer: &wgpu::Buffer, density_buffer: &wgpu::Buffer) {
        let mut new_num = num_new;
        if self.num_particles + num_new as u32 > self.max_particles as u32 {
            log::info!("Returning max particles instead of adding more");
            new_num = self.max_particles as u32 - self.num_particles
        }

        let mut new_particles = Vec::new();
        let mut new_positions = Vec::new();
        let mut new_velocities = Vec::new();
        let mut new_densities = Vec::new();

        for _ in 0..new_num {
            let x = rand::random::<f32>() * self.bound_size[0] - self.bound_size[0] / 2.0;
            let y = rand::random::<f32>() * self.bound_size[1] - self.bound_size[1] / 2.0;
            new_particles.push(ParticleLl::new(x, y));
            new_positions.push(PositionLl::new(x, y));
            new_velocities.push(VelocityLl::new());
            new_densities.push(DensityLl::new());

            self.particle_info.push(Particle::new(x, y, 1.0));
            self.particles.push(ParticleLl::new(x, y));
        }

        let offset = self.num_particles as wgpu::BufferAddress * std::mem::size_of::<ParticleLl>() as wgpu::BufferAddress;
        let offset_position = self.num_particles as wgpu::BufferAddress * std::mem::size_of::<PositionLl>() as wgpu::BufferAddress;
        let offset_velocity = self.num_particles as wgpu::BufferAddress * std::mem::size_of::<VelocityLl>() as wgpu::BufferAddress;
        let offset_density = self.num_particles as wgpu::BufferAddress * std::mem::size_of::<DensityLl>() as wgpu::BufferAddress;

        self.num_particles += new_num;
        let new_particle_data = bytemuck::cast_slice(&new_particles);
        let new_position_data = bytemuck::cast_slice(&new_positions);
        let new_velocity_data = bytemuck::cast_slice(&new_velocities);
        let new_density_data = bytemuck::cast_slice(&new_densities);

        queue.write_buffer(particle_buffer, offset, new_particle_data);
        queue.write_buffer(position_buffer, offset_position, new_position_data);
        queue.write_buffer(velocity_buffer, offset_velocity, new_velocity_data);
        queue.write_buffer(density_buffer, offset_density, new_density_data);        
    }
    
    //adds in a square spiral pattern, somwaht stupid ngl
    pub fn add_multiple_uniform_particles(&mut self, num_new: u32, queue: &wgpu::Queue, particle_buffer: &wgpu::Buffer, position_buffer: &wgpu::Buffer, velocity_buffer: &wgpu::Buffer, density_buffer: &wgpu::Buffer) {
        let mut new_num = num_new;
        if self.num_particles + num_new as u32 > self.max_particles as u32 {
            log::info!("Returning max particles instead of adding more");
            new_num = self.max_particles as u32 - self.num_particles;
        }

        let mut new_particles = Vec::new();

        let mut new_positions = Vec::new();
        let mut new_velocities = Vec::new();
        let mut new_densities = Vec::new();

        let mut pos = Vector2{x: 0.0, y:0.0};
        let mut delta_pos = Vector2{x: 0.0, y: 2.0*self.radius.radius};
        let rotation_matrix = cgmath::Matrix2::new(0.0, -1.0, 1.0, 0.0);


        let mut counter = 1;
        let mut segment_length = 1;
        let mut segment_counter = 1;

        for _ in 0..new_num {
            new_particles.push(ParticleLl::new(pos.x, pos.y));
            new_positions.push(PositionLl::new(pos.x, pos.y));
            new_velocities.push(VelocityLl::new());
            new_densities.push(DensityLl::new());




            self.particle_info.push(Particle::new(pos.x, pos.y, 1.0));
            self.particles.push(ParticleLl::new(pos.x, pos.y));

            if segment_counter == 0 {
                segment_counter = segment_length;
                delta_pos = rotation_matrix * delta_pos;
                counter -= 1;
            }

            if counter == 0 {
                counter = 2;
                segment_length += 1;
            }

            segment_counter -= 1;
            pos += delta_pos;
        }
        println!("particles: {:?}", new_particles.len());

        let offset = self.num_particles as wgpu::BufferAddress * std::mem::size_of::<ParticleLl>() as wgpu::BufferAddress;
        let offset_position = self.num_particles as wgpu::BufferAddress * std::mem::size_of::<PositionLl>() as wgpu::BufferAddress;
        let offset_velocity = self.num_particles as wgpu::BufferAddress * std::mem::size_of::<VelocityLl>() as wgpu::BufferAddress;
        let offset_density = self.num_particles as wgpu::BufferAddress * std::mem::size_of::<DensityLl>() as wgpu::BufferAddress;
        
        self.num_particles += new_num;
        let new_particle_data = bytemuck::cast_slice(&new_particles);
        let new_position_data = bytemuck::cast_slice(&new_positions);
        let new_velocity_data = bytemuck::cast_slice(&new_velocities);
        let new_density_data = bytemuck::cast_slice(&new_densities);

        queue.write_buffer(particle_buffer, offset, new_particle_data);
        queue.write_buffer(position_buffer, offset_position, new_position_data);
        queue.write_buffer(velocity_buffer, offset_velocity, new_velocity_data);
        queue.write_buffer(density_buffer, offset_density, new_density_data);
    }

    // pub fn calculate_density(&self, sample_point: Vector3<f32> ) -> f32 {
    //     let mut density = 0.0;

    //     for particle in self.particle_info.iter() {
    //         let distance = (sample_point - particle.position).magnitude();
    //         density += self.smoothing_kernel(self.smoothing_radius, distance);
    //     }

    //     density
    // }

    // //poly6 kernel
    // fn smoothing_kernel(&self, radius: f32, distance: f32) -> f32 {
    //     //let volume = 315.0 / (64.0 * std::f32::consts::PI * radius.powi(9));
    //     let volume = std::f32::consts::PI * radius.powi(4) / 6.0;
    //     let value = 0.0f32.max(radius - distance).powi(2);

    //     value / volume
    // }

    // fn smoothing_kernel_derivative(&self, radius: f32, distance: f32) -> f32 {

    //     if distance > radius { return 0.0; }
        
    //     let a = - 12.0 / (std::f32::consts::PI * radius.powi(4));
    //     let b =  radius - distance;
    //     a * b
    // }

    // pub fn update_density(&mut self) {
    //     for particle_index in 0..self.num_particles {
    //         let density = self.calculate_density(self.particle_info[particle_index as usize].position);
    //         self.particle_info[particle_index as usize].density = density;
    //     }
    // }
    

    // fn calclulate_pressure_force(&self, sample_point_index: u32, delta_time: Duration) -> Vector3<f32> {

    //     let mut pressure_force = Vector3::new(0.0, 0.0, 0.0);
    //     let sample_point = self.particle_info[sample_point_index as usize].position;

    //     for particle_index in 0..self.num_particles {
    //         if sample_point_index == particle_index { continue; }

    //         let particle = &self.particle_info[particle_index as usize];
    //         let distance = (particle.position - sample_point).magnitude();

    //         let direction;
    //         if distance == 0.0 {
    //             let x = rand::random::<f32>()* 2.0 - 1.0;
    //             let y = rand::random::<f32>()* 2.0 - 1.0;
    //             direction = Vector3::new(x, y, 0.0).normalize();
    //         } else {
    //             direction = (particle.position - sample_point) / distance;
    //         }

    //         let slope = self.smoothing_kernel_derivative(self.smoothing_radius, distance);
    //         let density = particle.density;
    //         let other_density = self.particle_info[sample_point_index as usize].density;
    //         let symmetric_pressure = self.calculate_symmetric_pressure(other_density, density);

    //         pressure_force += symmetric_pressure * direction * slope * particle.mass / density;
    //     }
    //     pressure_force
    // }

    // fn calculate_symmetric_pressure(&self, density_a: f32, density_b: f32) -> f32 {
    //     let pressure_a = self.convert_density_to_pressure(density_a);
    //     let pressure_b = self.convert_density_to_pressure(density_b);
    //     (pressure_a + pressure_b) / 2.0
    // }

    // fn convert_density_to_pressure(&self, density: f32) -> f32 {
    //     self.pressure_multiplier * (density - self.target_density)
    // }

}

// random info i want to use in shaders
#[repr(C)] // so stupid
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct RadiusLl {
    pub radius: f32,
}

impl RadiusLl {
    fn new(radius: f32) -> Self { Self { radius } }
}



#[derive(Debug, Copy, Clone)]
pub struct ParticleLl {
    pub position: Vector3<f32>, //12 bytes
    _padding: f32,
}

unsafe impl Pod for ParticleLl {}
unsafe impl Zeroable for ParticleLl {}

impl ParticleLl {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vector3 { x, y, z: 0.0 },
            _padding: 0.0,
        }
    }
}

pub struct Particle {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub mass: f32, //every particle has the same mass of 1.0
    pub density: f32,
}

impl Particle {
    pub fn new(x: f32, y: f32, mass: f32) -> Self {
        Self {
            position: Vector3 { x, y, z: 0.0 },
            velocity: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            mass,
            density: 0.1,
        }
    }

    pub fn apply_gravity(&mut self, delta_time: Duration) {
        return;
        let delta_seconds = delta_time.as_secs_f32();
        self.velocity.y -= GRAVITY * delta_seconds;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PositionLl {
    pub position: Vector3<f32>, //12 bytes
    _padding: f32,
}

unsafe impl Pod for PositionLl {}
unsafe impl Zeroable for PositionLl {}

impl PositionLl {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vector3 { x, y, z: 0.0 },
            _padding: 0.0,
        }
    }
}


#[derive(Debug, Copy, Clone)]
pub struct VelocityLl {
    pub position: Vector3<f32>, //12 bytes
    pub _padding: f32,
}

unsafe impl Pod for VelocityLl {}
unsafe impl Zeroable for VelocityLl {}

impl VelocityLl {
    pub fn new() -> Self {
        Self {
            position: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            _padding: 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DensityLl {
    pub density: Vector2<f32>,
}

unsafe impl Pod for DensityLl {}
unsafe impl Zeroable for DensityLl {}

impl DensityLl {
    pub fn new() -> Self {
        Self {
            density: Vector2 { x: 0.0, y: 0.0 },
        }
    }
}