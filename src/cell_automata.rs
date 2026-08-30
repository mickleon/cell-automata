#![doc = include_str!("../docs/CELL_AUTOMATA.md")]

use crate::cell_automata::Cell::*;
use std::fmt;

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

#[derive(Clone, Default, PartialEq, Eq)]
/// Cell of cellular automaton
pub enum Cell {
    Alive,
    #[default]
    Dead,
}

impl Cell {
    fn from(alive: bool) -> Self {
        if alive { Alive } else { Dead }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Alive => "[]",
            Dead => "__",
        };
        write!(f, "{}", symbol)
    }
}

/// Rules of cellular automaton
pub trait Rule {
    /// Returns an array of coordinates of the cell’s neighbours relative to it.
    fn neighbourhood(&self) -> &'static [(i8, i8)];
    /// Returns `true` if a live cell survives with a number of living neighbors `neighbors`.
    fn next_state(&self, cell: &Cell, neighbours: u8) -> Cell;
}

/// Rules of the Conway's Game of Life
pub struct ConwayRule;

impl Rule for ConwayRule {
    fn neighbourhood(&self) -> &'static [(i8, i8)] {
        &MOORE_NEIGHBOURHOOD_1
    }

    fn next_state(&self, cell: &Cell, neighbours: u8) -> Cell {
        Cell::from(matches!(
            (cell, neighbours),
            (Alive, 2) | (Alive, 3) | (Dead, 3)
        ))
    }
}

/// Custom rules of cellular automaton
pub struct CustomRule {
    /// An array of coordinates of the cell’s neighbours relative to it.
    pub neighbourhood: &'static [(i8, i8)],
    /// Number of living neighbourhs for cell birth.
    pub born: &'static [u8],
    /// Number of living neighbourhs for cell survive.
    pub survive: &'static [u8],
}

impl Rule for CustomRule {
    fn neighbourhood(&self) -> &'static [(i8, i8)] {
        self.neighbourhood
    }

    fn next_state(&self, cell: &Cell, neighbours: u8) -> Cell {
        let survives = *cell == Alive && self.survive.contains(&neighbours);
        let born = *cell == Dead && self.born.contains(&neighbours);
        Cell::from(survives || born)
    }
}

/// Cellular automaton with in size of `width * height`
#[derive(Default)]
pub struct CellAutomaton<R: Rule> {
    pub generation: u64,
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Cell>,
    rule: R,
}

impl<R: Rule> CellAutomaton<R> {
    /// Returns an automaton with all dead cells.
    pub fn new(width: usize, height: usize, rule: R) -> Self {
        let grid: Vec<Cell> = vec![Default::default(); width * height];
        CellAutomaton {
            generation: 0,
            width,
            height,
            grid,
            rule,
        }
    }

    pub fn randomize(&mut self, p: f64) {
        self.grid = (0..(self.width * self.height))
            .map(|_| Cell::from(rand::random_bool(p)))
            .collect();
    }

    pub fn clear(&mut self) {
        self.grid = vec![Default::default(); self.width * self.height];
    }

    fn neighbours(&self, x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> {
        self.rule.neighbourhood().iter().map(move |(dx, dy)| {
            (
                (x as i32 + *dx as i32).rem_euclid(self.width as i32) as usize,
                (y as i32 + *dy as i32).rem_euclid(self.height as i32) as usize,
            )
        })
    }

    /// Simualate an automaton's next generation. Returns a `false` if the automaton has finished working.
    /// ```
    /// # use cell_automata::cell_automata::{CellAutomaton, ConwayRule};
    /// let mut game_of_life = CellAutomaton::new(10, 10, ConwayRule);
    /// assert!(!game_of_life.step());
    /// ```
    pub fn step(&mut self) -> bool {
        let mut changes = Vec::with_capacity(self.width * self.height / 4);
        let mut idx = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                let neighbours_alive = self
                    .neighbours(x, y)
                    .filter(|(x, y)| *self.get(*x, *y) == Alive)
                    .count();

                let current_alive = &self.grid[idx];
                let new_alive = self.rule.next_state(current_alive, neighbours_alive as u8);

                if *current_alive != new_alive {
                    changes.push((idx, new_alive));
                }
                idx += 1;
            }
        }

        if changes.is_empty() {
            false
        } else {
            for (idx, new_cell) in changes {
                self.grid[idx] = new_cell;
            }
            self.generation += 1;
            true
        }
    }

    pub fn get(&self, x: usize, y: usize) -> &Cell {
        &self.grid[y * self.width + x]
    }
    pub fn set(&mut self, cell: Cell, x: usize, y: usize) {
        self.grid[y * self.width + x] = cell;
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
