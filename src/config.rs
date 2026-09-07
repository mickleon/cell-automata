use iced::Color;

pub const BACKGROUND_COLOR: Color = Color::BLACK;
pub const DEAD_COLOR: [u8; 4] = [255, 255, 255, 255];
pub const ALIVE_COLOR: [u8; 4] = [0, 0, 0, 255];

pub const DEFAULT_GRID_WIDTH: usize = 200;
pub const DEFAULT_GRID_HEIGHT: usize = 200;
pub const CEIL_ALIVE_PROBABILITY: f64 = 1.0 / 3.0;

pub const GEN_PER_SEC: f32 = 30.0;
pub const SPEED_SCALE: [f32; 6] = [0.25, 0.5, 1.0, 2.0, 4.0, 8.0];

pub const MIN_SCALE: f32 = 0.1;
pub const MAX_SCALE: f32 = 50.0;
pub const SCALE_STEP: f32 = 1.2;
