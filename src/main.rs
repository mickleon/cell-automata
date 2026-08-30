use iced::widget::{Action, Canvas, button, canvas, column, row};
use iced::{Color, Element, Event, Length, Point, Rectangle, Renderer, Theme, mouse};
use log::{debug, info};

pub fn main() -> iced::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_module("cell_automata", log::LevelFilter::Debug)
        .init();
    iced::application(Grid::default, Grid::update, Grid::view)
        .theme(Grid::theme)
        .run()
}

#[derive(Default)]
struct Grid {
    cache: canvas::Cache,
}

#[derive(Default)]
struct GridState {}

#[derive(Clone)]
enum Message {
    Clear,
    Randomize,
}

impl Grid {
    fn view(&self) -> Element<'_, Message> {
        column![
            Canvas::new(self).width(Length::Fill).height(Length::Fill),
            row![
                button("Clear").on_press(Message::Clear),
                button("Randomize").on_press(Message::Randomize),
            ]
            .spacing(20)
            .padding(10)
        ]
        .into()
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::Clear => {
                self.clear();
            }
            Message::Randomize => {
                self.randomize();
            }
        }
    }
    fn theme(&self) -> Theme {
        Theme::CatppuccinMacchiato
    }
    fn clear(&self) {
        info!("Grid clear");
        self.cache.clear();
    }
    fn randomize(&self) {
        info!("Grid randomize");
        self.cache.clear();
    }
}

impl canvas::Program<Message> for Grid {
    type State = GridState;

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                Action::<Message>::capture();
                None
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            debug!("Grid redraw");
            frame.fill_rectangle(Point::ORIGIN, frame.size(), Color::BLACK);
        });

        vec![geometry]
    }
}
