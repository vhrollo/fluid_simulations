// This is a simple 2D SPH simulation shader.

@compute @workgroup_size(16, 16, 1)
fn predict_position(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= num_particles) {
        return;
    }

    var pos = p_position[index].position;
    var vel = p_velocity[index].velocity;

    predicted_p_position[index].position = external_forces(&pos, &vel);
    p_velocity[index].velocity = vel;

}

@compute @workgroup_size(16, 16, 1)
fn calculate_density(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= num_particles) {
        return;
    }

    var density = update_density(index);
    p_density[index].density = density;
}

@compute @workgroup_size(16, 16, 1)
fn update_position(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= num_particles) {
        return;
    }

    var pos = p_position[index].position;
    var vel = p_velocity[index].velocity;
    let half_boundries = calculateBoundries();

    var density = p_density[index].density;
    var pressure_force = calculate_pressure_force(index, density);
    vel += pressure_force * delta_time;
    pos += vel * delta_time;

    checkBoundaries(&pos, &vel, half_boundries);

    p_position[index].position = pos;
    p_velocity[index].velocity = vel;
}


struct Particle_position {
    position: vec3f,
};

struct Particle_velocity {
    velocity: vec3f,
};

struct Particle_density {
    density: f32,
};

struct BoundryBox {
    boundry_box_center: vec2<f32>,
    boundry_box_size: vec2<f32>,
};

@group(0) @binding(0) var<storage, read_write> p_position: array<Particle_position>;
@group(0) @binding(1) var<storage, read_write> p_velocity: array<Particle_velocity>;
@group(0) @binding(2) var<storage, read_write> p_density: array<Particle_density>;
@group(0) @binding(3) var<storage, read_write> predicted_p_position: array<Particle_position>;

@group(1) @binding(0) var<uniform> radius: f32;
@group(1) @binding(1) var<uniform> num_particles: u32;
@group(1) @binding(2) var<storage, read> boundry_box: BoundryBox;
@group(1) @binding(3) var<uniform> delta_time: f32;


const PI: f32 = 3.14159265359;
const GRAVITY: f32 = 9.81;
const BOUNDARY_RESTITUTION: f32 = 0.95;
const SMOOTHING_RADIUS: f32 = 0.8;
const PRESSURE_MULTIPLIER: f32 = 2.0;
const TARGET_DENSITY: f32 = 10.0;
const TIME_STEP: f32 = 1 / 120.0;
const MASS: f32 = 1.0;

fn external_forces( pos: ptr<function, vec3<f32>>, vel: ptr<function, vec3<f32>>) -> vec3<f32> {
    (*vel).y -= GRAVITY * delta_time;
    return (*pos) + (*vel) * TIME_STEP;
}

fn calculateBoundries() -> vec2<f32> {
    return vec2(
        (boundry_box.boundry_box_size.x / 2.0) - radius,
        (boundry_box.boundry_box_size.y / 2.0) - radius
    );
}

// Simple boundary function
fn checkBoundaries(pos: ptr<function, vec3f>, vel: ptr<function, vec3f>, half_boundries: vec2<f32>) {
    let edge_dst = half_boundries - abs((*pos).xy - boundry_box.boundry_box_center.xy);
    if (edge_dst.x < 0.0) {
        (*pos).x = half_boundries.x * sign((*pos).x);
        (*vel).x = -(*vel).x * BOUNDARY_RESTITUTION;
    }

    if (edge_dst.y < 0.0) {
        (*pos).y = half_boundries.y * sign((*pos).y);
        (*vel).y = -(*vel).y * BOUNDARY_RESTITUTION;
    }
}

fn move_particle(pos: ptr<function, vec3<f32>>, vel: ptr<function, vec3<f32>>) {
    *pos += *vel;
    // *vel *= 0.0;
}

fn smoothing_kernel_spiky(s_rad: f32, dist: f32) -> f32 {
    if (dist > s_rad) { return 0.0; }

    var v: f32 = s_rad - dist;
    var volume: f32 = PI * pow(s_rad, 4.0) / 6.0;
    return v * v / volume;
}

fn smoothing_kernel_derivative(s_rad: f32, dist: f32) -> f32 {
    if (dist > s_rad) { return 0.0; }

    var volume: f32 = 12.0 / (PI * pow(s_rad, 4.0));
    var v: f32 = s_rad - dist;
    return - v * volume;
}

fn update_density(particle_index: u32) -> f32 {
    var density = 0.0;
    var particle_position = predicted_p_position[particle_index].position;
    var sqrRadius = SMOOTHING_RADIUS * SMOOTHING_RADIUS;

    for (var i: u32 = 0; i < num_particles; i++) {
        if (i == particle_index) {
            continue;
        }
        var neighbourPos = predicted_p_position[i].position;
        var offsetToNeighbour = neighbourPos - particle_position;
        var sqrDstToNeighbour = dot(offsetToNeighbour, offsetToNeighbour);

        if (sqrDstToNeighbour > sqrRadius) {
            continue;
        }

        var dist = sqrt(sqrDstToNeighbour);
        density += smoothing_kernel_spiky(SMOOTHING_RADIUS, dist); 
    }
    return max(density, 0.1);
}

fn calculate_symmetric_pressure(density_a: f32, density_b: f32) -> f32 {
    var pressure_a: f32 = convert_density_to_pressure(density_a);
    var pressure_b: f32 = convert_density_to_pressure(density_b);
    return (pressure_a + pressure_b) / 2.0;
}

fn convert_density_to_pressure(density: f32) -> f32 {
    return PRESSURE_MULTIPLIER * (density - TARGET_DENSITY);
}


fn calculate_pressure_force(particle_index: u32, density: f32) -> vec3f {
    var pressure_force = vec3f(0.0, 0.0, 0.0);
    var particle_position = predicted_p_position[particle_index].position;
    var sqrRadius = SMOOTHING_RADIUS * SMOOTHING_RADIUS;


    for (var i: u32 = 0; i < num_particles; i++) {
        if (i == particle_index) {
            continue;
        }

        var neighbourPos = predicted_p_position[i].position;
        var offsetToNeighbour = neighbourPos - particle_position;
        var sqrDstToNeighbour = dot(offsetToNeighbour, offsetToNeighbour);

        // Skip if not within radius
        if (sqrDstToNeighbour > sqrRadius) {continue;};

        // Calculate pressure force
        var dst = sqrt(sqrDstToNeighbour);
        var direction = normalize(offsetToNeighbour);
        if (dst < 0.01) {direction = get_random_direction(particle_index);};



        var slope = smoothing_kernel_derivative(SMOOTHING_RADIUS, dst);
        var other_particle_density = p_density[i].density;

        var symmetric_pressure = calculate_symmetric_pressure(other_particle_density, density);
        pressure_force += direction * slope * symmetric_pressure * MASS/ density;
    }

    return pressure_force;
}

// Other helper functions

fn lcg(seed: u32) -> u32 {
    let a: u32 = 1664525u;
    let c: u32 = 1013904223u;
    return a * seed + c;
}

fn random(seed: u32) -> f32 {
    let next_seed = lcg(seed);
    return fract(sin(f32(next_seed)) * 43758.5453123);
}

fn better_random(seed: u32) -> vec2<f32> {
    let s = lcg(seed);
    return vec2<f32>(random(s), random(lcg(s)));
}

fn get_random_direction(seed: u32) -> vec3<f32> {
    let rnd = better_random(seed);
    let theta = rnd.x * 2.0 * PI;
    let x = cos(theta);
    let y = sin(theta);
    return normalize(vec3<f32>(x, y, 0.0));
}
