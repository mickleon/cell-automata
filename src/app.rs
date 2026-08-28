use std::num::NonZeroU32;
use std::rc::Rc;

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

#[derive(Default)]
pub struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    frame: u32,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Cellular automaton")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
            .with_resizable(false);

        let window = Rc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Couldn't create a window"),
        );

        let context = Context::new(window.clone()).expect("Couldn't create a Context");
        let surface = Surface::new(&context, window.clone()).expect("Couldn't create a Surface");

        self.window = Some(window);
        self.surface = Some(surface);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(surface) = self.surface.as_mut()
                    && let (Some(width), Some(height)) =
                        (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                {
                    surface
                        .resize(width, height)
                        .expect("Couldn't resize the surface");
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.draw();
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn draw(&mut self) {
        let (Some(window), Some(surface)) = (&self.window, self.surface.as_mut()) else {
            return;
        };

        let size = window.inner_size();
        let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return;
        };

        surface
            .resize(width, height)
            .expect("Couldn't resize the surface");

        let mut buffer = surface.buffer_mut().expect("Couldn't get the buffer");

        let w = width.get();
        let h = height.get();

        for y in 0..h {
            for x in 0..w {
                let r: u32 = 255;
                let g: u32 = 255;
                let b: u32 = 255;

                let color = r << 16 | g << 8 | b;
                buffer[(y * w + x) as usize] = color;
            }
        }

        buffer.present().expect("Couldn't display buffer");
        self.frame = self.frame.wrapping_add(1);
    }
}
