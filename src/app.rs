use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

use crate::cell_automata::{CellAutomaton, ConwayRule};

const TARGET_FPS: f64 = 30.0;
const BACKGROUND_COLOR: u32 = 0x000000;
const ALIVE_COLOR: u32 = 0xffffff;
const DEAD_COLOR: u32 = 0x000000;

const MIN_SCALE: f32 = 0.1;
const MAX_SCALE: f32 = 50.0;
const ZOOM_STEP: f32 = 1.2;

pub struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    canvas: Canvas,

    dragging: bool,
    last_cursor_pos: Option<(f32, f32)>,
}

struct Canvas {
    automaton: CellAutomaton<ConwayRule>,
    field_left_top_x: i32,
    field_left_top_y: i32,
    field_scale: f32,
    x_start: u32,
    x_end: u32,
    y_start: u32,
    y_end: u32,
    width: u32,
    height: u32,
}

impl Canvas {
    /// Size (in pixels) that the automaton field occupies on screen at the
    /// current scale. Kept as a single helper so it's computed the same way
    /// everywhere instead of being duplicated across methods.
    fn dest_size(&self) -> (f32, f32) {
        (
            (self.automaton.width as f32 * self.field_scale).ceil(),
            (self.automaton.height as f32 * self.field_scale).ceil(),
        )
    }

    fn init(&mut self, width: u32, height: u32) {
        let (dest_width, dest_height) = self.dest_size();
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;

        self.field_left_top_x = (center_x - dest_width / 2.0) as i32;
        self.field_left_top_y = (center_y - dest_height / 2.0) as i32;
        self.resize(width, height);
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.recalc_bounds();
    }

    /// Recomputes the visible drawing window (`x_start..x_end`,
    /// `y_start..y_end`) from the current field position/scale and the
    /// window size. Clamping is done in `i32` before the cast to `u32`, so a
    /// field that is fully or partially off-screen (including when
    /// `field_left_top_x/y` is negative) never produces a wrapped/huge
    /// `u32` value.
    fn recalc_bounds(&mut self) {
        let (dest_width, dest_height) = self.dest_size();

        let x0 = self.field_left_top_x.clamp(0, self.width as i32);
        let y0 = self.field_left_top_y.clamp(0, self.height as i32);
        let x1 = (self.field_left_top_x + dest_width as i32).clamp(0, self.width as i32);
        let y1 = (self.field_left_top_y + dest_height as i32).clamp(0, self.height as i32);

        (self.x_start, self.y_start) = (x0 as u32, y0 as u32);
        (self.x_end, self.y_end) = (x1 as u32, y1 as u32);
    }

    /// Shifts the field by (dx, dy) screen pixels, e.g. while dragging.
    fn pan(&mut self, dx: f32, dy: f32) {
        self.field_left_top_x += dx.round() as i32;
        self.field_left_top_y += dy.round() as i32;
        self.recalc_bounds();
    }

    /// Multiplies the scale by `factor` (>1 zooms in, <1 zooms out), keeping
    /// the point under `(cursor_x, cursor_y)` (in screen coordinates) fixed
    /// on screen, so zooming feels anchored to the mouse instead of the
    /// window corner.
    fn zoom(&mut self, factor: f32, cursor_x: f32, cursor_y: f32) {
        let old_scale = self.field_scale;
        let new_scale = (old_scale * factor).clamp(MIN_SCALE, MAX_SCALE);
        if (new_scale - old_scale).abs() < f32::EPSILON {
            return;
        }

        let rel_x = cursor_x - self.field_left_top_x as f32;
        let rel_y = cursor_y - self.field_left_top_y as f32;
        let scale_ratio = new_scale / old_scale;

        self.field_left_top_x = (cursor_x - rel_x * scale_ratio) as i32;
        self.field_left_top_y = (cursor_y - rel_y * scale_ratio) as i32;
        self.field_scale = new_scale;

        self.recalc_bounds();
    }

    fn pixel_color(&self, pixel_x: i32, pixel_y: i32) -> Option<u32> {
        let cell_x = ((pixel_x - self.field_left_top_x) as f32 / self.field_scale) as i32;
        let cell_y = ((pixel_y - self.field_left_top_y) as f32 / self.field_scale) as i32;

        if cell_x < 0 || cell_y < 0 {
            return None;
        }
        let (src_x, src_y) = (cell_x as u32, cell_y as u32);

        if src_x >= self.automaton.width || src_y >= self.automaton.height {
            return None;
        }

        Some(if self.automaton.get(src_x, src_y).alive {
            ALIVE_COLOR
        } else {
            DEAD_COLOR
        })
    }

    fn draw_automata(&self, buffer: &mut softbuffer::Buffer<'_, Rc<Window>, Rc<Window>>) {
        for pixel_y in self.y_start..self.y_end {
            for pixel_x in self.x_start..self.x_end {
                if let Some(color) = self.pixel_color(pixel_x as i32, pixel_y as i32) {
                    buffer[((pixel_y * self.width) + pixel_x) as usize] = color;
                }
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: Default::default(),
            surface: Default::default(),
            canvas: Canvas {
                automaton: CellAutomaton::default().with_random(1.0 / 3.0),
                field_left_top_x: 0,
                field_left_top_y: 0,
                field_scale: 3.0,
                x_start: 0,
                x_end: 0,
                y_start: 0,
                y_end: 0,
                width: 0,
                height: 0,
            },
            dragging: false,
            last_cursor_pos: None,
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
        self.canvas.init(
            surface.window().inner_size().width,
            surface.window().inner_size().height,
        );

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
                    self.canvas.resize(width.get(), height.get());
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x as f32, position.y as f32);

                if self.dragging
                    && let Some((last_x, last_y)) = self.last_cursor_pos
                {
                    let (dx, dy) = (x - last_x, y - last_y);
                    if dx != 0.0 || dy != 0.0 {
                        self.canvas.pan(dx, dy);
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                }

                self.last_cursor_pos = Some((x, y));
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.dragging = state == ElementState::Pressed;
            }
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
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
            self.canvas.automaton.next_gen();
        }

        let next_frame_time = Instant::now() + Duration::from_secs_f64(1.0 / TARGET_FPS);
        event_loop.set_control_flow(ControlFlow::WaitUntil(next_frame_time));
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
