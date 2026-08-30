use std::rc::Rc;

use log::info;
use winit::window::Window;

use crate::cell_automata::{Cell, CellAutomaton, ConwayRule};
use crate::config::*;
use crate::utils::BidirectionalIter;

/// Current drawing status
pub enum DrawCell {
    Alive,
    Dead,
    False,
}

/// Main app part: canvas with displayed `automaton`
pub struct Canvas {
    pub automaton: CellAutomaton<ConwayRule>,
    pub speed: BidirectionalIter<'static, f32>,
    pub sim_paused: bool,
    pub drawing: DrawCell,
    pub width: u32,
    pub height: u32,

    field_left_top_x: i32,
    field_left_top_y: i32,
    field_scale: f32,
    x_start: u32,
    x_end: u32,
    y_start: u32,
    y_end: u32,
    x_cell_map: Vec<usize>,
    y_cell_map: Vec<usize>,
}

impl Default for Canvas {
    fn default() -> Self {
        let mut canvas = Canvas {
            automaton: CellAutomaton::new(200, 200, ConwayRule),
            speed: BidirectionalIter::new(&GENERATION_SPEED_FACTOR).with_pos(2),
            sim_paused: false,
            field_left_top_x: 0,
            field_left_top_y: 0,
            field_scale: 3.0,
            x_start: 0,
            x_end: 0,
            y_start: 0,
            y_end: 0,
            width: 0,
            height: 0,
            x_cell_map: Vec::new(),
            y_cell_map: Vec::new(),
            drawing: DrawCell::False,
        };
        canvas.randomize();
        canvas
    }
}

impl Canvas {
    /// Size (in pixels) that the automaton field occupies on screen at the
    /// current scale.
    fn dest_size(&self) -> (f32, f32) {
        (
            (self.automaton.width as f32 * self.field_scale).ceil(),
            (self.automaton.height as f32 * self.field_scale).ceil(),
        )
    }

    /// Align `autimaton`'s grid in window
    pub fn reset_transform(&mut self, width: u32, height: u32) {
        self.field_scale = (width as f32 / self.automaton.width as f32)
            .min(height as f32 / self.automaton.height as f32);
        let (dest_width, dest_height) = self.dest_size();
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;

        self.field_left_top_x = (center_x - dest_width / 2.0) as i32;
        self.field_left_top_y = (center_y - dest_height / 2.0) as i32;
        self.resize(width, height);
        info!("reset transform");
    }

    /// Resize canvas
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.recalc_bounds();
    }

    /// Recomputes the visible drawing window (`x_start..x_end`,
    /// `y_start..y_end`) from the current field position/scale and the
    /// window size.
    fn recalc_bounds(&mut self) {
        let (dest_width, dest_height) = (
            (self.automaton.width as f32 * self.field_scale).ceil(),
            (self.automaton.height as f32 * self.field_scale).ceil(),
        );

        let x0 = self.field_left_top_x.clamp(0, self.width as i32);
        let y0 = self.field_left_top_y.clamp(0, self.height as i32);
        let x1 = (self.field_left_top_x + dest_width as i32).clamp(0, self.width as i32);
        let y1 = (self.field_left_top_y + dest_height as i32).clamp(0, self.height as i32);

        (self.x_start, self.y_start) = (x0 as u32, y0 as u32);
        (self.x_end, self.y_end) = (x1 as u32, y1 as u32);

        self.rebuild_maps();
    }

    /// Recomputes `x_cell_map`/`y_cell_map` for the current visible range
    /// (`x_start..x_end`, `y_start..y_end`).
    fn rebuild_maps(&mut self) {
        self.x_cell_map.clear();
        self.x_cell_map
            .extend((self.x_start..self.x_end).map(|pixel_x| {
                ((pixel_x as i32 - self.field_left_top_x) as f32 / self.field_scale) as usize
            }));

        self.y_cell_map.clear();
        self.y_cell_map
            .extend((self.y_start..self.y_end).map(|pixel_y| {
                ((pixel_y as i32 - self.field_left_top_y) as f32 / self.field_scale) as usize
            }));
    }

    /// Randomize cells
    pub fn randomize(&mut self) {
        self.automaton.randomize(1.0 / 3.0);
        info!("randomized")
    }

    /// Маке all the cells dead
    pub fn clear(&mut self) {
        self.automaton.clear();
        info!("clear");
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.field_left_top_x += dx.round() as i32;
        self.field_left_top_y += dy.round() as i32;
        self.recalc_bounds();
    }

    /// Multiplies the scale by `factor` (>1 zooms in, <1 zooms out), keeping
    /// the point under `(cursor_x, cursor_y)` (in screen coordinates) fixed
    /// on screen.
    pub fn zoom(&mut self, factor: f32, cursor_x: f32, cursor_y: f32) {
        let old_scale = self.field_scale;
        let new_scale = (old_scale * factor).clamp(MIN_SCALE, MAX_SCALE);
        if (new_scale - old_scale).abs() < f32::EPSILON {
            return;
        }

        // Position of the cursor relative to the field's top-left corner,
        // before rescaling.
        let rel_x = cursor_x - self.field_left_top_x as f32;
        let rel_y = cursor_y - self.field_left_top_y as f32;
        let scale_ratio = new_scale / old_scale;

        self.field_left_top_x = (cursor_x - rel_x * scale_ratio) as i32;
        self.field_left_top_y = (cursor_y - rel_y * scale_ratio) as i32;
        self.field_scale = new_scale;

        self.recalc_bounds();
    }

    /// Draws the automaton using the precomputed `x_cell_map`/`y_cell_map`.
    pub fn draw_automata(&self, buffer: &mut softbuffer::Buffer<'_, Rc<Window>, Rc<Window>>) {
        for (row, &cell_y) in self.y_cell_map.iter().enumerate() {
            let pixel_y = self.y_start + row as u32;

            for (idx, &cell_x) in
                ((pixel_y * self.width + self.x_start) as usize..).zip(self.x_cell_map.iter())
            {
                let color = match self.automaton.get(cell_x, cell_y) {
                    Cell::Alive => ALIVE_COLOR,
                    Cell::Dead => DEAD_COLOR,
                };
                buffer[idx] = color;
            }
        }
    }

    /// Return target cell in automaton by displayed pixel coordinate
    fn get(&mut self, pixel_x: f32, pixel_y: f32) -> Option<(usize, usize)> {
        let pixel_x = pixel_x as u32;
        let pixel_y = pixel_y as u32;
        if self.x_start <= pixel_x
            && pixel_x <= self.x_end
            && self.y_start <= pixel_y
            && pixel_y <= self.y_end
        {
            let cell_x = self.x_cell_map[(pixel_x - self.x_start) as usize];
            let cell_y = self.y_cell_map[(pixel_y - self.y_start) as usize];

            return Some((cell_x, cell_y));
        }
        None
    }

    /// Enamble drawing mode
    pub fn start_draw(&mut self, pixel_x: f32, pixel_y: f32) {
        self.drawing = match self.get(pixel_x, pixel_y) {
            Some((x, y)) => match self.automaton.get(x, y) {
                Cell::Alive => {
                    self.automaton.set(Cell::Alive, x, y);
                    DrawCell::Dead
                }
                Cell::Dead => {
                    self.automaton.set(Cell::Dead, x, y);
                    DrawCell::Alive
                }
            },
            None => DrawCell::False,
        };
    }

    /// Should be called when mouse moved in drawing mode
    pub fn draw_cell(&mut self, pixel_x: f32, pixel_y: f32) {
        let status = match self.drawing {
            DrawCell::Alive => Some(Cell::Alive),
            DrawCell::Dead => Some(Cell::Dead),
            DrawCell::False => None,
        };
        let cell = self.get(pixel_x, pixel_y);
        if let Some((x, y)) = cell {
            self.automaton.set(status.unwrap(), x, y);
        };
    }

    /// Pause simulation
    pub fn sim_pause(&mut self) {
        self.sim_paused = true;
        info!("simulation paused");
    }

    /// Resume simulation
    pub fn sim_resume(&mut self) {
        self.sim_paused = false;
        info!("simulation resumed");
    }
}
