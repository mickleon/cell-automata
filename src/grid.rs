use std::time::Duration;

use iced::Alignment::Center;
use iced::widget::image;
use iced::widget::{Action, Canvas, button, canvas, column, row, text};
use iced::{Element, Event, Length, Point, Rectangle, Renderer, Subscription, Theme, mouse, time};
use log::{debug, info};

use crate::cell_automata::{Cell, CellAutomaton, ConwayRule};
use crate::config::*;
use crate::utils::BidirectionalIter;

#[derive(Clone, PartialEq)]
pub struct Grid {
    paused: bool,
    handle: image::Handle,
    automaton: CellAutomaton<ConwayRule>,
    speed_iter: BidirectionalIter<'static, f32>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Message {
    Step,
    TogglePause,
    Clear,
    Randomize,
    SpeedUp,
    SpeedDown,
}

impl Default for Grid {
    fn default() -> Self {
        let automaton = CellAutomaton::new(DEFAULT_GRID_HEIGHT, DEFAULT_GRID_WIDTH, ConwayRule);
        let mut grid = Self {
            paused: false,
            handle: Self::grid_handle(&automaton),
            automaton,
            speed_iter: BidirectionalIter::new(&SPEED_SCALE).with_pos(
                SPEED_SCALE
                    .iter()
                    .position(|x| *x == 1.0)
                    .unwrap_or(SPEED_SCALE.len() / 2),
            ),
        };
        grid.randomize();
        grid
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
                        .align_x(Center)
                        .align_y(Center),
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
                self.automaton.step();
                self.handle = Self::grid_handle(&self.automaton);
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
                self.handle = Self::grid_handle(&self.automaton);
            }
            Message::Randomize => {
                self.randomize();
                self.handle = Self::grid_handle(&self.automaton);
            }
            Message::SpeedUp => {
                self.speed_iter.next();
                info!("Speed up: {}x", *self.speed_iter)
            }
            Message::SpeedDown => {
                self.speed_iter.prev();
                info!("Speed down: {}x", *self.speed_iter)
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
        debug!("Grid recalc");
        image::Handle::from_rgba(automaton.width as u32, automaton.height as u32, buf)
    }
}

impl canvas::Program<Message> for Grid {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => None,
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
        frame.fill_rectangle(Point::ORIGIN, frame.size(), BACKGROUND_COLOR);
        frame.draw_image(
            Rectangle::new(Point::ORIGIN, frame.size()),
            canvas::Image::new(self.handle.clone()).filter_method(image::FilterMethod::Nearest),
        );
        vec![frame.into_geometry()]
    }
}
