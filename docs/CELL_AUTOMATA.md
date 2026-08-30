# cell-automata

A module for simulate cellular automata, in particular Conway's Game of Life.

## Example

```rust
use cell_automata::cell_automata::{CellAutomaton, ConwayRule};

fn main() {
    let mut game_of_life = CellAutomaton::new(20, 15, ConwayRule);

    game_of_life.randomize(1.0/3.0);

    for _ in 0..10 {
        print!("{}", game_of_life);
        if !game_of_life.step() {
            break;
        }
    }
}
```
