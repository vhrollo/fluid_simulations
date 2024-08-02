struct MatrixUniform {
    matrix: mat4x4<f32>,
};

@group(0) @binding(0) 
var<uniform> view : MatrixUniform;

@group(0) @binding(1) 
var<uniform> proj : MatrixUniform;

struct Particle_position {
    position: vec3<f32>,
};

@group(1) @binding(0) 
var<storage, read_write> p_position: array<Particle_position>;

struct Particle_velocity {
    velocity: vec3<f32>,
};

@group(1) @binding(1)
var<storage, read_write> p_velocity: array<Particle_velocity>;

struct Particle_density {
    density: f32,
};

@group(1) @binding(2)
var<storage, read_write> p_density: array<Particle_density>;

struct BoundryBox {
    center: vec2<f32>,
    size: vec2<f32>,
};

@group(2) @binding(0) var<uniform> radius: f32;
@group(2) @binding(1) var<uniform> num_particles: u32;
@group(2) @binding(2) var<storage, read> boundry_box: BoundryBox;


const OPENGL_TO_WGPU_MATRIX: mat4x4<f32> = mat4x4<f32>(
    vec4<f32>(1.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, 1.0, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 0.5, 0.5),
    vec4<f32>(0.0, 0.0, 0.0, 1.0)
);


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) fragCoords: vec2<f32>,
    @location(1) num_particles: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertexIndex: u32,
    @builtin(instance_index) instanceIndex: u32
) -> VertexOutput {

    var quadVertices: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0,  1.0)
    );

    var vertexId = vertexIndex % 4u;
    var instanceId = instanceIndex;


    var out: VertexOutput;
    var pos: vec2<f32> = ( boundry_box.size * quadVertices[vertexId] / 2.0 ) + boundry_box.center;
    out.clip_position = OPENGL_TO_WGPU_MATRIX * proj.matrix * view.matrix * vec4<f32>(pos, 0.0, 1.0);
    out.fragCoords = pos;
    return out; 
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var pos = in.fragCoords;
    var pressure = interpolation(pos);
    // var pressure = 1.0;
    return get_color(pressure);
}

const PI: f32 = 3.14159265359;
const SMOOTHING_RADIUS: f32 = 0.4;
const TARGET_DENSITY: f32 = 10.0;

fn get_color(pressure: f32) -> vec4<f32> {
    // Define the colors for the gradient
    let color_neg = vec4<f32>(0.0, 0.0, 1.0, 0.8); // Blue for very low pressure
    let color_low = vec4<f32>(0.0, 1.0, 1.0, 0.8); // Cyan for low pressure
    let color_target = vec4<f32>(1.0, 1.0, 1.0, 0.8); // White for target pressure
    let color_high = vec4<f32>(1.0, 1.0, 0.0, 0.8); // Yellow for high pressure
    let color_pos = vec4<f32>(1.0, 0.0, 0.0, 0.8); // Red for very high pressure

    var color: vec4<f32>;

    // Map pressure to color
    if (pressure < 0.0) {
        // Interpolate between blue and cyan for negative pressures
        let t = clamp((pressure + TARGET_DENSITY) / TARGET_DENSITY, 0.0, 1.0);
        color = mix(color_neg, color_low, t);
    } else {
        // Interpolate between white and red for positive pressures
        if (pressure < TARGET_DENSITY) {
            let t = clamp(pressure / TARGET_DENSITY, 0.0, 1.0);
            color = mix(color_low, color_target, t);
        } else {
            let t = clamp((pressure - TARGET_DENSITY) / TARGET_DENSITY, 0.0, 1.0);
            color = mix(color_target, color_high, t);
            color = mix(color_high, color_pos, t);
        }
    }

    return color;
}

fn interpolation(pos: vec2<f32>) -> f32 {
    var density = density_at_pos(pos);
    return density - TARGET_DENSITY; 
}

fn density_at_pos(particle_position: vec2<f32>) -> f32 {
    var density = 0.0;
    var sqrRadius = SMOOTHING_RADIUS * SMOOTHING_RADIUS;

    for (var i: u32 = 0; i < num_particles; i++) {
        var neighbourPos = p_position[i].position.xy;
        // var neighbourPos = vec2<f32>(0.0,0.0);
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

fn smoothing_kernel_spiky(s_rad: f32, dist: f32) -> f32 {
    if (dist > s_rad) { return 0.0; }

    var v: f32 = s_rad - dist;
    var volume: f32 = PI * pow(s_rad, 4.0) / 6.0;
    return v * v / volume;
}

// fm smoothing_kernel_spikey_near(s_rad: f32, dist: f32) -> f32 {
//     if (dist > s_rad) { return 0.0; }

//     var v: f32 = s_rad - dist;
//     var volume: f32 = PI * pow(s_rad, 4.0) / 6.0;
//     return v * v / volume;
// }