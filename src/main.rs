use std::{thread::sleep, time::Duration};

use cell_automata::{CellAutomaton, ConwayRule};

fn main() {
    let mut game_of_life = CellAutomaton::from_random(20, 15, 1.0 / 3.0, ConwayRule);

    loop {
        sleep(Duration::from_millis(100));
        print!("{}", game_of_life);
        if !game_of_life.next_gen() {
            break;
        }
    }
}
