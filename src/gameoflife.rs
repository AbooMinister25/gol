use serde::{Deserialize, Serialize};

use std::default::Default;
use std::fmt;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use crate::app::{MessageState, Pattern};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Universe {
    #[serde(skip)]
    since: Instant,
    pub width: u32,
    pub height: u32,
    pub interval: u32,
    pub cells: Vec<Cell>,
    pub started: bool,
}

impl Universe {
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        sender: &Sender<String>,
        receiver: &Receiver<MessageState>,
    ) {
        // Receive a message and match on the contents.
        sender.send(self.to_string()).expect("Error while sending");
        if let Ok(message) = receiver.try_recv() {
            match message {
                MessageState::Pattern(p) => match p {
                    Pattern::Spaceship => self.set_cells(&[(1, 2), (2, 3), (3, 1), (3, 2), (3, 3)]),
                },
                MessageState::Start => self.started = true,
                MessageState::Pause => self.started = false,
                MessageState::Clear => {
                    let len = 0..self.width * self.height;
                    self.cells = vec![Cell::Dead; len.len()];
                }
            }
        }

        // Play one generation of the game.
        if self.started {
            self.tick();
            sender.send(self.to_string()).expect("error while sending");
            ctx.request_repaint();
        }
    }

    pub fn tick(&mut self) {
        let mut next_generation = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, neighbors) {
                    // If a cell has less than two live neighbors, it dies.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // If a cell has two or three live neighbors, it lives.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // If a cell has more than three neighbors, it dies.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // If a dead cell has exactly three live neighbors, it becomes a live cell.
                    (Cell::Dead, 3) => Cell::Alive,
                    // Otherwise, just leave it as it is.
                    (cell, _) => cell,
                };

                next_generation[idx] = next_cell;
            }
        }

        self.cells = next_generation;
    }

    fn get_index(&self, row: u32, col: u32) -> usize {
        (row * self.width + col) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;

        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;

                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }

        count
    }

    fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
        }
    }
}

impl Default for Universe {
    fn default() -> Self {
        let width = 64;
        let height = 64;

        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Self {
            since: Instant::now(),
            width,
            height,
            interval: 50,
            cells,
            started: false,
        }
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
