use std::time::Duration;

use iced::widget::{Canvas, button, canvas, column, image, row, text};
use iced::{
    Alignment, Element, Event, Length, Point, Rectangle, Renderer, Size, Subscription, Theme,
    mouse, time, widget,
};
use log::{debug, error, info};

use crate::cell_automata::{Cell, CellAutomaton, ConwayRule};
use crate::config::*;
use crate::utils::BidirectionalIter;

#[derive(Clone, PartialEq)]
pub struct Grid {
    automaton: CellAutomaton<ConwayRule>,
    handle: image::Handle,

    scale: f32,
    translation: iced::Vector,

    paused: bool,
    speed_iter: BidirectionalIter<'static, f32>,
}

struct ViewTransform {
    fit_scale: f32,
    image_size: Size,
    image_origin: Point,
    center: iced::Vector,
}

#[derive(Clone, PartialEq)]
pub enum Message {
    Step,
    TogglePause,
    Clear,
    Randomize,
    SpeedUp,
    SpeedDown,
    Pan(iced::Vector),
    Zoom {
        delta: f32,
        cursor: Point,
        canvas_size: iced::Size,
    },
    Draw {
        coord: (usize, usize),
        cell: Cell,
    },
}

impl Default for Grid {
    fn default() -> Self {
        let automaton = CellAutomaton::new(DEFAULT_GRID_HEIGHT, DEFAULT_GRID_WIDTH, ConwayRule);
        Self {
            handle: Self::grid_handle(&automaton),
            automaton,
            scale: 1.0,
            translation: iced::Vector::new(0.0, 0.0),
            paused: true,
            speed_iter: BidirectionalIter::new(&SPEED_SCALE).with_pos(
                SPEED_SCALE
                    .iter()
                    .position(|x| *x == 1.0)
                    .unwrap_or(SPEED_SCALE.len() / 2),
            ),
        }
    }
}

impl Grid {
    pub fn view(&self) -> Element<'_, Message> {
        let pause_button_icon = if self.paused {
            "\u{f040a}"
        } else {
            "\u{f03e4}"
        };
        column![
            row![
                row![
                    button(pause_button_icon)
                        .width(30)
                        .on_press(Message::TogglePause),
                    button("\u{f045f}").on_press(Message::SpeedDown),
                    text(format!("{}x", *self.speed_iter))
                        .width(50)
                        .height(30)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                    button("\u{f0211}").on_press(Message::SpeedUp),
                ]
                .spacing(10),
                row![
                    button("Randomize").on_press(Message::Randomize),
                    button("Clear").on_press(Message::Clear),
                ]
                .spacing(10),
            ]
            .spacing(40)
            .padding(10),
            Canvas::new(self).width(Length::Fill).height(Length::Fill),
        ]
        .into()
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Step => {
                if !self.automaton.step() {
                    self.pause();
                }
                self.update_handle();
            }
            Message::TogglePause => {
                if self.paused {
                    self.resume();
                } else {
                    self.pause();
                }
            }
            Message::Clear => {
                self.clear();
                self.pause();
                self.update_handle();
            }
            Message::Randomize => {
                self.randomize();
                self.update_handle();
            }
            Message::SpeedUp => {
                self.speed_iter.next();
                info!("Speed up: {}x", *self.speed_iter)
            }
            Message::SpeedDown => {
                self.speed_iter.prev();
                info!("Speed down: {}x", *self.speed_iter)
            }
            Message::Pan(delta) => {
                self.pan(delta);
            }
            Message::Zoom {
                delta,
                cursor,
                canvas_size,
            } => {
                self.zoom(delta, cursor, canvas_size);
            }
            Message::Draw { coord, cell } => {
                let (x, y) = coord;
                // debug!("Draw {}, {}: {:?}", coord.0, coord.1, cell);
                match self.automaton.set(cell, x, y) {
                    Ok(_) => {
                        self.update_handle();
                    }
                    Err(_) => {
                        error!(
                            "Failed to draw a cell ({}, {}) because of out of bounds",
                            x, y
                        );
                    }
                }
            }
        }
    }
    pub fn theme(&self) -> Theme {
        Theme::CatppuccinMacchiato
    }
    pub fn automaton_step_sub(&self) -> Subscription<Message> {
        if self.paused {
            return Subscription::none();
        }

        let gen_per_sec = DEFAULT_GEN_PER_SEC * *self.speed_iter;
        let millis = (1000.0 / gen_per_sec) as u64;

        time::every(Duration::from_millis(millis)).map(|_| Message::Step)
    }
    fn clear(&mut self) {
        self.automaton.clear();
        info!("Grid clear");
    }
    fn randomize(&mut self) {
        self.automaton.randomize(CEIL_ALIVE_PROBABILITY);
        info!("Grid randomize");
    }
    fn pause(&mut self) {
        self.paused = true;
        info!("Paused");
    }
    fn resume(&mut self) {
        self.paused = false;
        info!("Resume");
    }
    fn pan(&mut self, delta: iced::Vector) {
        self.translation += delta * (1.0 / self.scale);
        // debug!("Pan: {}, {}", self.translation.x, self.translation.y);
    }
    fn zoom(&mut self, delta: f32, cursor: Point, canvas_size: iced::Size) {
        if delta == 0.0 {
            return;
        }

        let zoom_factor = if delta > 0.0 {
            ZOOM_STEP
        } else {
            1.0 / ZOOM_STEP
        };
        let old_scale = self.scale;
        let new_scale = (old_scale * zoom_factor).clamp(MIN_SCALE, MAX_SCALE);

        if new_scale == old_scale {
            return;
        }

        let center = Point::new(canvas_size.width / 2.0, canvas_size.height / 2.0);
        self.translation += (cursor - center) * (1.0 / new_scale - 1.0 / old_scale);
        self.scale = new_scale;
        // debug!("Zoom: {}x", self.scale);
    }
    fn grid_handle(automaton: &CellAutomaton<ConwayRule>) -> image::Handle {
        let mut buf = Vec::with_capacity(automaton.grid.len() * 4);
        for cell in &automaton.grid {
            let color = if *cell == Cell::Alive {
                ALIVE_COLOR
            } else {
                DEAD_COLOR
            };
            buf.extend_from_slice(&color); // RGBA
        }
        // debug!("Grid rebuild");
        image::Handle::from_rgba(automaton.width as u32, automaton.height as u32, buf)
    }
    fn update_handle(&mut self) {
        self.handle = Self::grid_handle(&self.automaton);
    }
    fn view_transform(&self, bounds: Rectangle) -> ViewTransform {
        let center = iced::Vector::new(bounds.width / 2.0, bounds.height / 2.0);
        let grid_size = Size::new(self.automaton.width as f32, self.automaton.height as f32);
        let fit_scale = (bounds.width / grid_size.width).min(bounds.height / grid_size.height);
        let image_size = Size::new(grid_size.width * fit_scale, grid_size.height * fit_scale);
        let image_origin = Point::new(
            (bounds.width - image_size.width) / 2.0,
            (bounds.height - image_size.height) / 2.0,
        );
        ViewTransform {
            fit_scale,
            image_size,
            image_origin,
            center,
        }
    }
    fn screen_to_cell(&self, point: Point, bounds: Rectangle) -> Option<(usize, usize)> {
        let vt = self.view_transform(bounds);

        let mut v = iced::Vector::new(point.x - vt.center.x, point.y - vt.center.y);
        v /= self.scale;
        v += vt.center - self.translation;
        let ix = (v.x - vt.image_origin.x) / vt.fit_scale;
        let iy = (v.y - vt.image_origin.y) / vt.fit_scale;

        if ix < 0.0 || iy < 0.0 {
            return None;
        };

        let x = ix as usize;
        let y = iy as usize;
        if x < self.automaton.width && y < self.automaton.height {
            Some((x, y))
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct GridState {
    panning: bool,
    drawing: Option<Cell>,
    cursor_last: Option<Point>,
}

impl canvas::Program<Message> for Grid {
    type State = GridState;
    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<widget::Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.panning = true;
                state.cursor_last = None;
                state.drawing = None;
                None
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.panning = false;
                None
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                let point = cursor.position_in(bounds)?;
                let (x, y) = self.screen_to_cell(point, bounds)?;
                let clicked_cell = self.automaton.get_unchecked(x, y);
                let new_cell = match clicked_cell {
                    Cell::Alive => Cell::Dead,
                    Cell::Dead => Cell::Alive,
                };

                state.drawing = Some(new_cell);
                state.cursor_last = None;

                Some(widget::Action::publish(Message::Draw {
                    coord: (x, y),
                    cell: new_cell,
                }))
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) => {
                state.drawing = None;
                None
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let pos = cursor.position_in(bounds)?;

                let action = if state.panning {
                    state
                        .cursor_last
                        .map(|last| widget::Action::publish(Message::Pan(pos - last)))
                } else if let Some(cell) = state.drawing {
                    self.screen_to_cell(pos, bounds)
                        .map(|coord| widget::Action::publish(Message::Draw { coord, cell }))
                } else {
                    None
                };

                state.cursor_last = Some(pos);
                action
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let zoom_factor = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => y / 60.0,
                };

                let cursor_pos = cursor
                    .position_in(bounds)
                    .or(state.cursor_last)
                    .unwrap_or(Point::ORIGIN);

                Some(widget::Action::publish(Message::Zoom {
                    delta: zoom_factor,
                    cursor: cursor_pos,
                    canvas_size: bounds.size(),
                }))
            }
            _ => None,
        }
    }
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let vt = self.view_transform(bounds);

        frame.fill_rectangle(Point::ORIGIN, frame.size(), BACKGROUND_COLOR);

        frame.with_save(|frame| {
            frame.translate(vt.center);
            frame.scale(self.scale);
            frame.translate(self.translation - vt.center);

            frame.draw_image(
                Rectangle::new(vt.image_origin, vt.image_size),
                canvas::Image::new(&self.handle).filter_method(image::FilterMethod::Nearest),
            );
        });

        vec![frame.into_geometry()]
    }
    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.panning {
            mouse::Interaction::Grabbing
        } else if state.drawing.is_some() {
            mouse::Interaction::Idle
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }
}
