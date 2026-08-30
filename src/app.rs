use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};

use log::info;
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::{CursorIcon, Window, WindowId};

use crate::canvas::{Canvas, DrawCell};
use crate::config::*;

pub struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    next_frame_time: Instant,
    canvas: Canvas,

    dragging: bool,
    last_cursor_pos: Option<(f32, f32)>,
    modifiers: ModifiersState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: Default::default(),
            surface: Default::default(),
            next_frame_time: Instant::now(),
            canvas: Default::default(),
            dragging: false,
            last_cursor_pos: None,
            modifiers: ModifiersState::default(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Cellular automaton");

        let window = Rc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Couldn't create a window"),
        );

        let context = Context::new(window.clone()).expect("Couldn't create a Context");
        let surface = Surface::new(&context, window.clone()).expect("Couldn't create a Surface");
        self.canvas.reset_transform(
            surface.window().inner_size().width,
            surface.window().inner_size().height,
        );

        self.window = Some(window);
        self.surface = Some(surface);
        self.canvas.randomize();
        info!("app resumed")
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
                    self.canvas.resize(width.get(), height.get());
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x as f32, position.y as f32);
                let mut redraw: bool = false;
                match self.canvas.drawing {
                    DrawCell::False => {}
                    _ => {
                        self.canvas.draw_cell(x, y);
                        redraw = true;
                    }
                }

                if self.dragging
                    && let Some((last_x, last_y)) = self.last_cursor_pos
                {
                    let (dx, dy) = (x - last_x, y - last_y);
                    if dx != 0.0 || dy != 0.0 {
                        self.canvas.pan(dx, dy);
                        redraw = true;
                    }
                }
                if redraw && let Some(window) = &self.window {
                    window.request_redraw();
                }

                self.last_cursor_pos = Some((x, y));
            }
            // Left mouse key
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.dragging = state == ElementState::Pressed;
                if let Some(window) = &self.window {
                    window.set_cursor(if self.dragging {
                        CursorIcon::Grabbing
                    } else {
                        CursorIcon::Default
                    });
                }
            }
            // Right mouse key
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Right,
                ..
            } => match state {
                ElementState::Pressed => {
                    if let Some((x, y)) = self.last_cursor_pos {
                        self.canvas.start_draw(x, y);
                    }
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                ElementState::Released => {
                    self.canvas.drawing = DrawCell::False;
                }
            },
            // Mouse wheel
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => (pos.y / 60.0) as f32,
                };

                if scroll_y != 0.0 {
                    let zoom_factor = if scroll_y > 0.0 {
                        ZOOM_STEP
                    } else {
                        1.0 / ZOOM_STEP
                    };

                    let (cursor_x, cursor_y) = self.last_cursor_pos.unwrap_or((
                        self.canvas.width as f32 / 2.0,
                        self.canvas.height as f32 / 2.0,
                    ));

                    self.canvas.zoom(zoom_factor, cursor_x, cursor_y);

                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::ModifiersChanged(new_mods) => {
                self.modifiers = new_mods.state();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key
                    && event.state.is_pressed()
                    && !event.repeat
                {
                    match code {
                        KeyCode::KeyP => {
                            if self.canvas.sim_paused {
                                // Shift + P
                                self.canvas.sim_resume();
                            } else {
                                // Key P
                                self.canvas.sim_pause();
                            }
                        }
                        KeyCode::KeyR => {
                            if self.modifiers.shift_key() {
                                // Shift + R
                                self.canvas.randomize();
                            } else {
                                // Key R
                                self.canvas
                                    .reset_transform(self.canvas.width, self.canvas.height);
                            }
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
                        KeyCode::KeyC => {
                            if self.modifiers.shift_key() {
                                // Shift + C
                                self.canvas.clear();
                            }
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
                        // Arrow Up
                        KeyCode::ArrowUp => {
                            if let Some(s) = self.canvas.speed.next() {
                                info!("speed: {}x", s);
                            }
                        }
                        // Arrow Down
                        KeyCode::ArrowDown => {
                            if let Some(s) = self.canvas.speed.prev() {
                                info!("speed: {}x", s);
                            }
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        if !self.canvas.sim_paused && now >= self.next_frame_time {
            if !self.canvas.automaton.step() {
                self.canvas.sim_pause();
            }
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            self.next_frame_time =
                now + Duration::from_secs_f32(1.0 / (GENERATION_SPEED * *self.canvas.speed.get()));
        }

        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame_time));
    }
}

impl App {
    fn draw(&mut self) {
        let Some(surface) = self.surface.as_mut() else {
            return;
        };

        let mut buffer = surface.buffer_mut().unwrap();
        buffer.fill(BACKGROUND_COLOR);
        self.canvas.draw_automata(&mut buffer);

        buffer.present().unwrap();
    }
}
