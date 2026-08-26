#![doc = include_str!("../README.md")]
use std::{fmt, iter};

/// Moore neighbourhood of range 1
/// ```text
/// [ ][ ][ ][ ][ ]
/// [ ][@][@][@][ ]
/// [ ][@][*][@][ ]
/// [ ][@][@][@][ ]
/// [ ][ ][ ][ ][ ]
/// ```
#[rustfmt::skip]
pub static MOORE_NEIGHBOURHOOD_1: [(i8, i8); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1), (0, 1),
    (1, -1), (1, 0), (1, 1),
];

/// Von Neumann neighbourhood of range 1
/// ```text
/// [ ][ ][ ][ ][ ]
/// [ ][ ][@][ ][ ]
/// [ ][@][*][@][ ]
/// [ ][ ][@][ ][ ]
/// [ ][ ][ ][ ][ ]
/// ```
#[rustfmt::skip]
pub static NEUMANN_NEIGHBOURHOOD_1: [(i8, i8); 4] = [
    (-1, 0),
    (0, -1), (0, 1),
    (1, 0),
];

#[derive(Clone)]
/// Cell of cellular automaton
pub struct Cell {
    alive: bool,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = if self.alive { "[]" } else { "__" };
        write!(f, "{}", symbol)
    }
}

/// Rules of cellular automaton
pub trait Rule {
    /// Returns an array of coordinates of the cell’s neighbours relative to it.
    fn neighbourhood(&self) -> &'static [(i8, i8)];
    /// Returns `true` if a live cell survives with a number of living neighbors `neighbors`.
    fn next_state(&self, alive: bool, neighbours: i8) -> bool;
}

/// Rules of the Conway's Game of Life
pub struct ConwayRule;

impl Rule for ConwayRule {
    fn neighbourhood(&self) -> &'static [(i8, i8)] {
        &MOORE_NEIGHBOURHOOD_1
    }

    fn next_state(&self, alive: bool, neighbours: i8) -> bool {
        matches!((alive, neighbours), (true, 2) | (true, 3) | (false, 3))
    }
}

/// Cellular automaton with in size of `width * height`
pub struct CellAutomaton<R: Rule> {
    pub generation: u64,
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Cell>,
    rule: R,
}

impl<R: Rule> CellAutomaton<R> {
    /// Returns an automaton with random distribution of live cells with probability `p`.
    pub fn from_random(width: usize, height: usize, p: f64, rule: R) -> Self {
        let grid = (0..(width * height))
            .map(|_| Cell {
                alive: rand::random_bool(p),
            })
            .collect();
        CellAutomaton {
            generation: 0,
            width,
            height,
            grid,
            rule,
        }
    }

    /// Returns an automaton with all dead cells.
    pub fn from_blank(width: usize, height: usize, rule: R) -> Self {
        let grid = Vec::from_iter(iter::repeat_n(Cell { alive: false }, width * height));
        CellAutomaton {
            generation: 0,
            width,
            height,
            grid,
            rule,
        }
    }

    /// Simualate an automaton's next generation. Returns a `false` if the automaton has finished working.
    /// ```
    /// # use cell_automata::{CellAutomaton, ConwayRule};
    /// let mut game_of_life = CellAutomaton::from_blank(10, 10, ConwayRule);
    /// assert!(!game_of_life.next_gen());
    pub fn next_gen(&mut self) -> bool {
        let mut changes = Vec::with_capacity(self.width * self.height / 4);
        let neighbourhood = self.rule.neighbourhood();

        for x in 0..self.width {
            for y in 0..self.height {
                let idx = self.width * y + x;
                let mut neighbours_alive = 0;

                for &(dx, dy) in neighbourhood {
                    let nx = (x as i32 + dx as i32).rem_euclid(self.width as i32) as usize;
                    let ny = (y as i32 + dy as i32).rem_euclid(self.height as i32) as usize;
                    if self.grid[self.width * ny + nx].alive {
                        neighbours_alive += 1;
                    }
                }

                let current_alive = self.grid[idx].alive;
                let new_alive = self.rule.next_state(current_alive, neighbours_alive);

                if current_alive != new_alive {
                    changes.push((idx, new_alive));
                }
            }
        }

        if changes.is_empty() {
            false
        } else {
            for (idx, alive) in changes {
                self.grid[idx].alive = alive;
            }
            self.generation += 1;
            true
        }
    }
}

impl<R: Rule> fmt::Display for CellAutomaton<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for counter in 0..(self.height * self.width) {
            write!(f, "{}", self.grid[counter])?;
            if counter % self.width == self.width - 1 {
                writeln!(f)?;
            }
        }
        writeln!(f, "Generation: {}", self.generation)?;
        Ok(())
    }
}

/// Custom rules of cellular automaton
pub struct CustomRule {
    /// An array of coordinates of the cell’s neighbours relative to it.
    pub neighbourhood: &'static [(i8, i8)],
    /// Number of living neighbourhs for cell birth.
    pub born: &'static [i8],
    /// Number of living neighbourhs for cell survive.
    pub survive: &'static [i8],
}

impl Rule for CustomRule {
    fn neighbourhood(&self) -> &'static [(i8, i8)] {
        self.neighbourhood
    }

    fn next_state(&self, alive: bool, neighbours: i8) -> bool {
        let survives = alive && self.survive.contains(&neighbours);
        let born = !alive && self.born.contains(&neighbours);
        survives || born
    }
}
