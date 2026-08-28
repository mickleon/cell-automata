# cell-automata

A module for simulate cellular automata, in particular Conway's the Game of Life.

## Example

```rust
use cell_automata::{CellAutomaton, ConwayRule};

fn main() {
    let mut game_of_life = CellAutomaton::from_random(20, 15, 1.0 / 3.0, ConwayRule);

    for _ in 0..10 {
        print!("{}", game_of_life);
        if !game_of_life.next_gen() {
            break;
        }
    }
}
```
