# 2D SPH Particle Simulation

A simple 2D Smoothed Particle Hydrodynamics (SPH) simulation using WGSL, and some GLSL (as i switched mid-project), and Rust. The simulation models the behavior of particles under fluid dynamics, providing visualizations and interactions with a GPU-based approach. ( don't mind how messy it is, it's still work in progress :D )

## Features

- GPU-accelerated particle simulation.
- Real-time visualization of particle movements.
- Multi-pass computation for accurate particle interaction.
- Interaction with particles

## Sample
![](https://github.com/vhrollo/fluid_simulations/blob/main/example/example.gif)

## To do

- [ ] Extending into 3D
- [ ] Optimize even more
- [ ] Ray Marching

## Sources
- https://github.com/SebLague/Fluid-Sim
- https://web.archive.org/web/20140725014123/https://docs.nvidia.com/cuda/samples/5_Simulations/particles/doc/particles.pdf
- https://sph-tutorial.physics-simulation.org/pdf/SPH_Tutorial.pdf
- http://www.ligum.umontreal.ca/Clavet-2005-PVFS/pvfs.pdf
- https://matthias-research.github.io/pages/publications/sca03.pdf