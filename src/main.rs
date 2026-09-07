use cell_automata::grid::Grid;

pub fn main() -> iced::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_module("cell_automata", log::LevelFilter::Debug)
        .init();
    iced::application(Grid::default, Grid::update, Grid::view)
        .theme(Grid::theme)
        .subscription(Grid::timer_sub)
        .run()
}
