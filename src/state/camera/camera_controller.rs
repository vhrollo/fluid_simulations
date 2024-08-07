#![feature(duration_millis_float)]
use std::time::{Duration, Instant};

use cgmath::Vector2;
use winit::{event::{WindowEvent::{self, KeyboardInput}, ElementState, KeyEvent}, keyboard::{KeyCode, PhysicalKey}, dpi::PhysicalPosition};
use super::camera::{ViewMatrix, CameraMatrix};
use fluid_simulations::SVec;
use bytemuck::{Pod, Zeroable};


#[repr(C, align(4))]
#[derive(Debug, Copy, Clone)]
pub struct MouseDelta {
    pub previous_position: Vector2<f32>,
    pub current_position: Vector2<f32>,
}
unsafe impl Pod for MouseDelta {}
unsafe impl Zeroable for MouseDelta {}


pub struct CameraController {
    speed: f32,
    sensitivity: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    pub is_mouse_pressed: bool,
    previous_mouse_position: Option<PhysicalPosition<f64>>,
    last_frame_time: Instant,
    yaw: f32,
    pitch: f32,
    x_delta: f32,
    y_delta: f32,
    pub mouse_delta: MouseDelta,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32, view: &ViewMatrix) -> Self {

        let (yaw, pitch) = view.start_yaw_and_pitch;


        Self {
            speed,
            sensitivity,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_mouse_pressed: false,
            previous_mouse_position: None,
            last_frame_time: Instant::now(),
            yaw,
            pitch,
            x_delta: 0.0,
            y_delta: 0.0,
            mouse_delta: MouseDelta {
                previous_position: Vector2::new(1000.0, 1000.0),
                current_position: Vector2::new(1000.0, 1000.0),
            },
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent, size: winit::dpi::PhysicalSize<u32>) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    KeyCode::ControlLeft => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    KeyCode::ShiftLeft => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    _ => false
                }
            }
            WindowEvent::MouseInput { state, .. } => {
                self.is_mouse_pressed = *state == ElementState::Pressed;
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.is_mouse_pressed {
                    if let Some(prev_pos) = self.previous_mouse_position {
                        self.x_delta += (position.x - prev_pos.x) as f32;
                        self.y_delta += (position.y - prev_pos.y) as f32;
                    }
                    self.mouse_delta.previous_position = self.mouse_delta.current_position;
                    self.mouse_delta.current_position = Vector2::new(
                        ((position.x as f32 / size.width as f32) * 2.0 - 1.0),
                        -(((position.y as f32 / size.height as f32) * 2.0 - 1.0))
                    );
                    // println!("{:?}", self.mouse_delta);
                    self.previous_mouse_position = Some(*position);
                } else {
                    self.previous_mouse_position = Some(*position);
                }
                true
            }
            _ => false,
        }
    }

    pub fn yaw_pitch(&mut self, delta_time: Duration) {
        let delta_x = self.x_delta;
        let delta_y = self.y_delta;
        self.x_delta = 0.0;
        self.y_delta = 0.0;

        let yaw = delta_x * delta_time.as_secs_f32() * self.sensitivity;
        let pitch = delta_y * delta_time.as_secs_f32() * self.sensitivity;

        self.yaw -= yaw;
        self.pitch -= pitch;
        self.pitch = self.pitch.clamp(-89.0_f32, 89.0_f32);
    }	

    pub fn update_camera(&self, camera: &mut ViewMatrix, delta_time: Duration) {
        camera.forward = [ //yaw pitch rotation
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        ];

        let f = { //norm
            let f = &camera.forward;
            let len = f[0]*f[0] + f[1]*f[1] + f[2] * f[2];
            let len = len.sqrt();
            [f[0] / len, f[1] / len, f[2] / len]
        };
        
        let up = &camera.up;

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

        if self.is_forward_pressed { // cursed makeshift type SVec<f32>
            camera.position += &(&SVec { elements: f.to_vec() } * (self.speed * delta_time.as_secs_f32()));
        }
        if self.is_backward_pressed {
            camera.position -= &(&SVec { elements: f.to_vec() } * (self.speed * delta_time.as_secs_f32()));
        }
        if self.is_right_pressed {
            camera.position += &(&SVec { elements: s_norm.to_vec() } * (self.speed * delta_time.as_secs_f32()));
        }
        if self.is_left_pressed {
            camera.position -= &(&SVec { elements: s_norm.to_vec() } * (self.speed * delta_time.as_secs_f32()));
        }
        if self.is_up_pressed {
            camera.position += &(&SVec { elements: u.to_vec() } * (self.speed * delta_time.as_secs_f32()));
        }
        if self.is_down_pressed {
            camera.position -= &(&SVec { elements: u.to_vec() } * (self.speed * delta_time.as_secs_f32()));
        }

    }
}
