// This is a simple 2D SPH simulation shader.

@compute @workgroup_size(16, 16, 1)
fn predict_position(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var index = global_id.x;
    if (index >= num_particles) {
        return;
    }

    var pos = p_position[index].position;
    var vel = p_velocity[index].velocity;

    predicted_p_position[index].position = external_forces(&pos, &vel);
    p_velocity[index].velocity = vel;

}



@compute @workgroup_size(16, 16, 1)
fn update_spatial_hash(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var index = global_id.x;
    if (index >= num_particles) {
        return;
    }
    var half_boundries = calculateBoundries();
    var pos = predicted_p_position[index].position;
    var norm_pos = get_shifted_2D_pos(pos.xy, half_boundries);
    var hash_key = hash_position(norm_pos);

    spatial_hash[index].cell_key = hash_key;
    spatial_hash[index].particle_index = index;
}    

// black magic fr, this alternative non-recursive method of bitonic sort is a bit more complex
//https://en.m.wikipedia.org/wiki/Bitonic_sorter
@compute @workgroup_size(16, 16, 1)
fn bitonic_sort_kernel(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var index = global_id.x;
    var l = index ^ c.j;

    if (index >= c.next_pwr) {
        return;
    }

    if (l > index) {
        var a = spatial_hash[index];
        var b = spatial_hash[l];
        if ((index & c.k) == 0) {
            if (a.cell_key > b.cell_key) {
                spatial_hash[index] = b;
                spatial_hash[l] = a;
            }
        } else {
            if (a.cell_key < b.cell_key) {
                spatial_hash[index] = b;
                spatial_hash[l] = a;
            }
        }
    }
    

    workgroupBarrier(); // Ensure all threads have completed their operations before next iteration
}

@compute @workgroup_size(16, 16, 1)
fn reset_indecies(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var index = global_id.x;
    if (index >= max_particles) {
        return;
    }
    start_indices[index] = max_particles;
}

@compute @workgroup_size(16, 16, 1)
fn calculate_start_indices(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var index = global_id.x;
    if (index >= num_particles) {
        return;
    }

    var key = spatial_hash[index].cell_key;
    if (index == 0) {
        start_indices[key] = 0u;
    } else if key != spatial_hash[index - 1].cell_key {
        start_indices[key] = index;
    }

}

@compute @workgroup_size(16, 16, 1)
fn calculate_density(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= num_particles) {
        return;
    }

    var density = fast_update_density(index);
    p_density[index].density = density;
}

@compute @workgroup_size(16, 16, 1)
fn calculate_viscosity(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= num_particles) {
        return;
    }

    var viscosity_force = fast_calculate_viscosity_force(index);
    // p_velocity[index].velocity += viscosity_force * VISCOSITY_STRENGTH * delta_time;
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
    var pressure_force = fast_calculate_pressure_force(index, density);
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

struct KeyValuePair {
    particle_index: u32,
    cell_key: i32,
};

@group(0) @binding(0) var<storage, read_write> p_position: array<Particle_position>;
@group(0) @binding(1) var<storage, read_write> p_velocity: array<Particle_velocity>;
@group(0) @binding(2) var<storage, read_write> p_density: array<Particle_density>;
@group(0) @binding(3) var<storage, read_write> predicted_p_position: array<Particle_position>;

@group(1) @binding(0) var<uniform> radius: f32;
@group(1) @binding(1) var<uniform> num_particles: u32;
@group(1) @binding(2) var<storage, read> boundry_box: BoundryBox;
@group(1) @binding(3) var<uniform> delta_time: f32;
@group(1) @binding(4) var<uniform> max_particles: u32;

@group(2) @binding(0) var<storage, read_write> spatial_hash: array<KeyValuePair>;
@group(2) @binding(1) var<storage, read_write> start_indices: array<u32>;
@group(2) @binding(1) var<storage, read_write> entries: array<u32>;

struct PushConstants { k: u32, j: u32, next_pwr: u32 }
var<push_constant> c: PushConstants;

const PI: f32 = 3.14159265359;
const GRAVITY: f32 = 9.81;
const BOUNDARY_RESTITUTION: f32 = 0.95;
const SMOOTHING_RADIUS: f32 = 0.4;
const PRESSURE_MULTIPLIER: f32 = 1.3;
const TARGET_DENSITY: f32 = 10.0;
const TIME_STEP: f32 = 1 / 120.0;
const TIME_STEP_SQ: f32 = TIME_STEP * TIME_STEP;
const MASS: f32 = 1.0;
const VISCOSITY_STRENGTH: f32 = 0.1;

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
    var edge_dst = half_boundries - abs((*pos).xy - boundry_box.boundry_box_center.xy);
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

fn smoothing_kernel_poly6(s_rad: f32, dist: f32) -> f32 {
    if (dist > s_rad) { return 0.0; }

    var v: f32 = pow(s_rad * s_rad - dist * dist, 3.0);
    var volume: f32 = 315.0 / (64.0 * PI * pow(s_rad, 9.0));
    return volume * v;
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

fn fast_update_density(particle_index: u32) -> f32 {
    var density = 0.0;
    var particle_position = predicted_p_position[particle_index].position;
    var norm_particle_position = get_shifted_2D_pos(particle_position.xy, calculateBoundries());
    var sqrRadius = SMOOTHING_RADIUS * SMOOTHING_RADIUS;
    var neighbor_offsets_2D = array<vec2<i32>, 9>(
        vec2<i32>(-1, -1), vec2<i32>(0, -1), vec2<i32>(1, -1),
        vec2<i32>(-1, 0), vec2<i32>(0, 0), vec2<i32>(1, 0),
        vec2<i32>(-1, 1), vec2<i32>(0, 1), vec2<i32>(1, 1)
    );

    for (var i: u32 = 0; i < 9; i++) {
        var pos_offset = norm_particle_position + neighbor_offsets_2D[i];   
        var hash_key = hash_position(pos_offset);

        if hash_key < 0 || hash_key >= i32(max_particles) {
            continue;
        }

        var curr_index = start_indices[hash_key];

        while (curr_index < max_particles && spatial_hash[curr_index].cell_key == hash_key) {
            var neighbour_index = spatial_hash[curr_index].particle_index;

            if (neighbour_index >= num_particles) {
                break;
            }

            if (neighbour_index == particle_index) {
                curr_index += 1u;
                continue;
            }
            var neighbour_pos = predicted_p_position[neighbour_index].position;
            var offset_to_neighbour = neighbour_pos - particle_position;
            var sqr_dst_to_neighbour = dot(offset_to_neighbour, offset_to_neighbour);

            if (sqr_dst_to_neighbour > sqrRadius) {
                curr_index += 1u;
                continue;
            }

            var dist = sqrt(sqr_dst_to_neighbour);
            density += smoothing_kernel_spiky(SMOOTHING_RADIUS, dist); 
        
            curr_index += 1u;
        }
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

fn fast_calculate_pressure_force(particle_index: u32, density: f32) -> vec3f {
    var pressure_force = vec3f(0.0, 0.0, 0.0);
    var particle_position = predicted_p_position[particle_index].position;
    var norm_particle_position = get_shifted_2D_pos(particle_position.xy, calculateBoundries());
    var sqrRadius = SMOOTHING_RADIUS * SMOOTHING_RADIUS;
    var neighbor_offsets_2D = array<vec2<i32>, 9>(
        vec2<i32>(-1, -1), vec2<i32>(0, -1), vec2<i32>(1, -1),
        vec2<i32>(-1, 0), vec2<i32>(0, 0), vec2<i32>(1, 0),
        vec2<i32>(-1, 1), vec2<i32>(0, 1), vec2<i32>(1, 1)
    );

    for (var i: u32 = 0; i < 9; i++) {
        var pos_offset = norm_particle_position + neighbor_offsets_2D[i];   
        var hash_key = hash_position(pos_offset);

        if hash_key < 0 || hash_key >= i32(max_particles) {
            continue;
        }

        var curr_index = start_indices[hash_key];

        while (curr_index < max_particles && spatial_hash[curr_index].cell_key == hash_key) {
            var neighbour_index = spatial_hash[curr_index].particle_index;

            if (neighbour_index >= num_particles) {
                break;
            }

            if (neighbour_index == particle_index) {
                curr_index += 1u;
                continue;
            }
            var neighbour_pos = predicted_p_position[neighbour_index].position;
            var offset_to_neighbour = neighbour_pos - particle_position;
            var sqr_dst_to_neighbour = dot(offset_to_neighbour, offset_to_neighbour);

            if (sqr_dst_to_neighbour > sqrRadius) {
                curr_index += 1u;
                continue;
            }

            var dst = sqrt(sqr_dst_to_neighbour);
            var direction = normalize(offset_to_neighbour);
            if (dst < 0.01) {
                direction = get_random_direction(particle_index);
            }

            var slope = smoothing_kernel_derivative(SMOOTHING_RADIUS, dst);
            var other_particle_density = p_density[neighbour_index].density;

            var symmetric_pressure = calculate_symmetric_pressure(other_particle_density, density);
            pressure_force += direction * slope * symmetric_pressure * MASS / density;
        
            curr_index += 1u;
        }
    }
    return pressure_force;
}

// Other helper functions

fn lcg(seed: u32) -> u32 {
    var a: u32 = 1664525u;
    var c: u32 = 1013904223u;
    return a * seed + c;
}

fn random(seed: u32) -> f32 {
    var next_seed = lcg(seed);
    return fract(sin(f32(next_seed)) * 43758.5453123);
}

fn better_random(seed: u32) -> vec2<f32> {
    var s = lcg(seed);
    return vec2<f32>(random(s), random(lcg(s)));
}

fn get_random_direction(seed: u32) -> vec3<f32> {
    var rnd = better_random(seed);
    var theta = rnd.x * 2.0 * PI;
    var x = cos(theta);
    var y = sin(theta);
    return normalize(vec3<f32>(x, y, 0.0));
}

// Spatial hash functions


const hv1: i32 = 73856093;
const hv2: i32 = 19349663;
const hv3: i32 = 83492791; // for 3d hashing

fn get_shifted_2D_pos(pos: vec2<f32>, half_boundries: vec2<f32>) -> vec2<i32> {
    var floor_x = i32( floor( (pos.x + half_boundries.x) / SMOOTHING_RADIUS ) );
    var floor_y = i32( floor( (pos.y + half_boundries.y) / SMOOTHING_RADIUS ) );
    return vec2<i32>(floor_x, floor_y);
}


// seems funky to use a float as a hash value, but it works for now
fn hash_position(pos: vec2<i32>) -> i32 {
    // var hash = (p.x * hv1 + p.y * hv2); //if wrapping behavior isnt supported
    //seems to give more uniform distribution
    var hash = (pos.x * hv1) ^ (pos.y * hv2); //if wrapping behavior is supported
    return hash % i32(max_particles);
}

fn fast_calculate_viscosity_force(index: u32) -> vec3f{
    var viscosity_force = vec3f(0.0, 0.0, 0.0);
    var particle_position = predicted_p_position[index].position;
    var norm_particle_position = get_shifted_2D_pos(particle_position.xy, calculateBoundries());
    var sqrRadius = SMOOTHING_RADIUS * SMOOTHING_RADIUS;
    var neighbor_offsets_2D = array<vec2<i32>, 9>(
        vec2<i32>(-1, -1), vec2<i32>(0, -1), vec2<i32>(1, -1),
        vec2<i32>(-1, 0), vec2<i32>(0, 0), vec2<i32>(1, 0),
        vec2<i32>(-1, 1), vec2<i32>(0, 1), vec2<i32>(1, 1)
    );

    for (var i: u32 = 0; i < 9; i++) {
        var pos_offset = norm_particle_position + neighbor_offsets_2D[i];   
        var hash_key = hash_position(pos_offset);

        if hash_key < 0 || hash_key >= i32(max_particles) {
            continue;
        }

        var curr_index = start_indices[hash_key];

        while (curr_index < max_particles && spatial_hash[curr_index].cell_key == hash_key) {
            var neighbour_index = spatial_hash[curr_index].particle_index;

            if (neighbour_index >= num_particles) {
                break;
            }

            if (neighbour_index == index) {
                curr_index += 1u;
                continue;
            }
            var neighbour_pos = predicted_p_position[neighbour_index].position;
            var offset_to_neighbour = neighbour_pos - particle_position;
            var sqr_dst_to_neighbour = dot(offset_to_neighbour, offset_to_neighbour);

            if (sqr_dst_to_neighbour > sqrRadius) {
                curr_index += 1u;
                continue;
            }

            var dist = sqrt(sqr_dst_to_neighbour);
            var vel_diff = p_velocity[neighbour_index].velocity - p_velocity[index].velocity;
            var direction = normalize(offset_to_neighbour);
            var laplacian = smoothing_kernel_poly6(SMOOTHING_RADIUS, dist);
            viscosity_force += vel_diff * laplacian * MASS;

            curr_index += 1u;
        }
    }
    return viscosity_force;
}