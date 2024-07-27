pub mod utils;
pub mod state;
pub mod simulation;


use winit::{event_loop::EventLoop, window::WindowBuilder};
use state::{State, ApplicationEvent};

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(
            env_logger::DEFAULT_FILTER_ENV, "info, wgpu_hal::vulkan::instance=warn"
        )
    );
    #[allow(deprecated)]
    let event_loop = EventLoop::<ApplicationEvent>::with_user_event().expect("event loop building");
    let window = WindowBuilder::new().build(&event_loop).expect("window building");
    
    let state = futures::executor::block_on(State::new(&window));
    state.run(event_loop);
}