use winit::event_loop::{ControlFlow, EventLoop};

use cell_automata::app::App;

fn main() {
    let event_loop = EventLoop::new().expect("Couldn't create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .expect("Application execution error");
}
