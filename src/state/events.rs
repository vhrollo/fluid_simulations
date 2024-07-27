use winit::{event::{ElementState, Event, WindowEvent}, event_loop::{ControlFlow, EventLoopWindowTarget}, keyboard::{KeyCode, PhysicalKey::Code}};
use crate::State;
use crate::state::update::OtherLogic;
use crate::state::render::Render;

#[derive(Debug)]
pub enum ApplicationEvent {
    #[allow(unused)]
    Crash,
}

pub enum Update {
    Continuous,
    #[allow(unused)]
    Reactive, 
}

pub trait EventHandler {
    fn handle_event(&mut self, event: &WindowEvent) -> bool;
    fn process_event_loop(&mut self, event: Event<ApplicationEvent>, control_flow: &EventLoopWindowTarget<ApplicationEvent>);
}

impl<'a> EventHandler for State<'a> {
    fn handle_event(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    fn process_event_loop(&mut self, event: Event<ApplicationEvent>, control_flow: &EventLoopWindowTarget<ApplicationEvent>) {
        control_flow.set_control_flow(ControlFlow::Wait);
            
        //println!("event: {:?}", &event);
        match event {
            Event::UserEvent(event) => match event {
                ApplicationEvent::Crash => {
                    control_flow.exit();
                }
            }
            Event::WindowEvent { event, .. } => {
                if !self.input(&event) {
                    match event {                    
                        WindowEvent::CloseRequested => {
                            control_flow.exit();
                            self.console_logger.cleanup();
                        }
                        WindowEvent::Resized(physical_size) => {
                            self.size = physical_size;
                            log::info!("Resized to {:?}", physical_size);
                            self.surface_configured = true;
                            self.resize(physical_size);
                        }
                        WindowEvent::Moved(_) => {
                            self.window.request_redraw();
                        }
                        WindowEvent::KeyboardInput {
                            event: winit::event::KeyEvent { 
                                state: ElementState::Pressed, 
                                physical_key: Code(KeyCode::Space),
                                ..
                            }, 
                            is_synthetic: false, 
                            ..
                        } => {
                            self.space = !self.space;
                        }
                        WindowEvent::KeyboardInput {
                            event: winit::event::KeyEvent { 
                                state: ElementState::Pressed, 
                                physical_key: Code(KeyCode::KeyK),
                                ..
                            }, 
                            is_synthetic: false, 
                            ..
                        } => {
                            self.paused = !self.paused;
                        }
                        WindowEvent::KeyboardInput {
                            event: winit::event::KeyEvent { 
                                state: ElementState::Pressed, 
                                physical_key: Code(KeyCode::KeyO),
                                ..
                            }, 
                            is_synthetic: false, 
                            ..
                        } => {
                            self.water_simulation.smoothing_radius += 0.01;
                        }
                        WindowEvent::RedrawRequested => {
                            if !self.surface_configured {
                                return;
                            } 
                            match self.update_mode {
                                Update::Continuous => {
                                    self.update();
                                    self.timestamp();
                                },
                                Update::Reactive => {
                                    if !self.should_update() {
                                        return;
                                    }
                                    self.timestamp();
                                    self.update();
                                }
                            }
                            match self.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => {
                                    self.surface.configure(&self.device, &self.config);
                                    self.surface_configured = false;
                                }
                                #[allow(unreachable_patterns)]
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    self.resize(self.size)
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    log::error!("Out of memory");
                                    control_flow.exit();
                                }
                                Err(wgpu::SurfaceError::Timeout) => {
                                    log::error!("Timeout");
                                }

                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::AboutToWait => {
                self.window.request_redraw();
            }
            _ => {}
        }
    }
}
