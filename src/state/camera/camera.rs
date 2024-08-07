use cgmath::{Matrix3, SquareMatrix};
use fluid_simulations::SVec;


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MatrixUniform {
    pub matrix: [[f32; 4]; 4],
}


pub struct CameraMatrix {
    fov: f32,
    znear: f32,
    zfar: f32,
    pub camera_matrix: MatrixUniform,
}

pub struct ViewMatrix {
    pub position: SVec<f32>,
    pub forward: [f32; 3],
    pub up: [f32; 3],
    pub view_matrix: MatrixUniform,
    pub start_yaw_and_pitch: (f32, f32),
}

impl CameraMatrix {
    pub fn new(size: winit::dpi::PhysicalSize<u32>, fov: f32, znear: f32, zfar: f32) -> Self {
        let aspect = size.height as f32 / size.width as f32;
        let matrix = Self::calculate_matrix(aspect, fov, znear, zfar);

        Self {
            fov,
            znear,
            zfar,
            camera_matrix: MatrixUniform { matrix },
        }
    }

    pub fn update_proj(&mut self,size: winit::dpi::PhysicalSize<u32>) {
        let aspect = size.height as f32 / size.width as f32;
        let matrix = Self::calculate_matrix(aspect, self.fov, self.znear, self.zfar);
        self.camera_matrix = MatrixUniform { matrix };
    }

    fn calculate_matrix(aspect: f32, fov: f32, znear: f32, zfar: f32) -> [[f32; 4];4] {
        let fov = fov * std::f32::consts::PI /180.0; //fov should be given in degrees for more ease
        let f = 1.0 / (fov/2.0).tan(); //given that the screen is 2 wide h/2 equals 1

        [
            [    f *   aspect     ,    0.0,              0.0              ,   0.0],
            [         0.0         ,     f ,              0.0              ,   0.0],
            [         0.0         ,    0.0,  (zfar+znear)/(zfar-znear)    ,   1.0],
            [         0.0         ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0f32],
        ]
    }
}


impl ViewMatrix {
    pub fn new(position: [f32; 3], up: [f32; 3], yaw: f32, pitch: f32) -> Self {
        let position = SVec {elements: position.to_vec()};
        let pitch = pitch.clamp(-89.0_f32, 89.0_f32);

        let forward = [
            yaw.to_radians().cos() * pitch.to_radians().cos(),
            pitch.to_radians().sin(),
            yaw.to_radians().sin() * pitch.to_radians().cos(),
        ];
        let matrix = Self::calculate_matrix(&position, forward, up);

        Self {
            position,
            forward,
            up, 
            view_matrix: MatrixUniform {matrix},
            start_yaw_and_pitch: (yaw, pitch),
        }
    }

    pub fn update_view(&mut self) {
        let matrix = Self::calculate_matrix(&self.position, self.forward, self.up);
        self.view_matrix = MatrixUniform {matrix};
    }

    fn calculate_matrix(position: &SVec<f32>, forward: [f32; 3], up: [f32; 3],) -> [[f32; 4];4] {
        let f = { //norm
            let f = forward;
            let len = f[0]*f[0] + f[1]*f[1] + f[2] * f[2];
            let len = len.sqrt();
            [f[0] / len, f[1] / len, f[2] / len]
        };

        let s = [ //cross product to get size vector to left of f and up
            up[1] * f[2] - up[2] * f[1],
            up[2] * f[0] - up[0] * f[2],
            up[0] * f[1] - up[1] * f[0]
        ];

        let s_norm = { // norm
            let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
            let len = len.sqrt();
            [s[0] / len, s[1] / len, s[2] / len]
        };

        let u = [ // ensuring orthogonality with the up vector
            f[1] * s_norm[2] - f[2] * s_norm[1],
            f[2] * s_norm[0] - f[0] * s_norm[2],
            f[0] * s_norm[1] - f[1] * s_norm[0]
        ];

        let p = [ // change of basis
            -position.elements[0] * s_norm[0] - position.elements[1] * s_norm[1] - position.elements[2] * s_norm[2],
            -position.elements[0] * u[0] - position.elements[1] * u[1] - position.elements[2] * u[2],
            -position.elements[0] * f[0] - position.elements[1] * f[1] - position.elements[2] * f[2]
        ];

        [  // view matrix
            [s_norm[0], u[0], f[0], 0.0],
            [s_norm[1], u[1], f[1], 0.0],
            [s_norm[2], u[2], f[2], 0.0],
            [p[0], p[1], p[2], 1.0],
        ]
    }
}


// some custom type shit to make vector adding easier and vector mul

pub fn inverse(matrix: cgmath::Matrix4<f32>) -> cgmath::Matrix4<f32> {
    let inverted = matrix.invert().unwrap();
    inverted
}