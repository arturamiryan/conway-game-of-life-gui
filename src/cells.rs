use std::cell::RefCell;

use crate::{HEIGHT, WIDTH};

#[derive(Clone, Copy)]
struct Cell {
    alive: bool,
    color: [u8; 4],
}

impl Cell {
    pub fn is_alive(self) -> bool {
        self.alive
    }

    pub fn set_alive(&mut self) {
        self.alive = true;
    }

    pub fn set_dead(&mut self) {
        self.alive = false;
    }

    pub fn set_color(&mut self, color: [u8; 4]) {
        self.color = color;
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            alive: false,
            color: [0, 0xff, 0, 0xf0],
        }
    }
}

pub struct Grid {
    cells: Vec<Vec<Cell>>,
    swap_cells: Vec<Vec<Cell>>,
    cells_color: [u8; 4],
}

pub struct Grid_color<'a> {
    owner: &'a RefCell<Grid>,
}

impl<'a> Grid_color<'a> {
    fn get_mut_color(&self) -> [u8; 4] {
        self.owner.borrow().cells_color
    }
}

impl Grid {
    pub fn new(width: u32, height: u32, cells_color: [u8; 4]) -> Grid {
        Self {
            cells: vec![
                vec![
                    Cell {
                        alive: false,
                        ..Default::default()
                    };
                    width as usize
                ];
                height as usize
            ],
            swap_cells: vec![
                vec![
                    Cell {
                        alive: false,
                        ..Default::default()
                    };
                    width as usize
                ];
                height as usize
            ],
            cells_color,
        }
    }

    pub fn set_cells_color(&mut self, cells_color: [u8; 4]) {
        self.cells_color = cells_color;
    }

    pub fn get_mut_cells_color(&mut self) -> &mut [u8; 4] {
        &mut self.cells_color
    }

    pub fn randomize(&mut self) {
        for i in 0..HEIGHT as usize {
            for j in 0..WIDTH as usize {
                self.cells[i][j] = Cell {
                    alive: rand::random(),
                    color: self.cells_color,
                };
            }
        }
    }

    pub fn set_blank(&mut self) {
        for i in 0..HEIGHT as usize {
            for j in 0..WIDTH as usize {
                self.cells[i][j] = Cell {
                    alive: false,
                    color: self.cells_color,
                };
            }
        }
    }

    pub fn draw(&self, screen: &mut [u8]) {
        let mut pix = screen.chunks_exact_mut(4);
        for row in self.cells.iter() {
            for cell in row {
                let color = if cell.is_alive() {
                    cell.color
                } else {
                    [0, 0xff, 0x01, 0x05]
                };
                pix.next().unwrap().copy_from_slice(&color);
            }
        }
    }

    pub fn check_rules(&mut self) {
        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                self.swap_cells[i as usize][j as usize] = Cell {
                    alive: self.decide(i, j),
                    color: self.cells_color,
                };
            }
        }
        std::mem::swap(&mut self.swap_cells, &mut self.cells);
    }

    pub fn set_pixel_alive(&mut self, x: f32, y: f32) {
        self.cells[y as usize][x as usize].set_alive();
    }

    pub fn set_pixel_dead(&mut self, x: f32, y: f32) {
        self.cells[y as usize][x as usize].set_dead();
    }

    // Total neighbourgs of (i, j) cell
    fn get_neighbours(&self, i: i32, j: i32) -> usize {
        self.cells[((i - 1 + HEIGHT) % HEIGHT) as usize][((j - 1 + WIDTH) % WIDTH) as usize]
            .is_alive() as usize
            + self.cells[((i - 1 + HEIGHT) % HEIGHT) as usize][((j + WIDTH) % WIDTH) as usize]
                .is_alive() as usize
            + self.cells[((i - 1 + HEIGHT) % HEIGHT) as usize][((j + 1 + WIDTH) % WIDTH) as usize]
                .is_alive() as usize
            + self.cells[((i + HEIGHT) % HEIGHT) as usize][((j - 1 + WIDTH) % WIDTH) as usize]
                .is_alive() as usize
            + self.cells[((i + HEIGHT) % HEIGHT) as usize][((j + 1 + WIDTH) % WIDTH) as usize]
                .is_alive() as usize
            + self.cells[((i + 1 + HEIGHT) % HEIGHT) as usize][((j - 1 + WIDTH) % WIDTH) as usize]
                .is_alive() as usize
            + self.cells[((i + 1 + HEIGHT) % HEIGHT) as usize][((j + WIDTH) % WIDTH) as usize]
                .is_alive() as usize
            + self.cells[((i + 1 + HEIGHT) % HEIGHT) as usize][((j + 1 + WIDTH) % WIDTH) as usize]
                .is_alive() as usize
    }

    fn decide(&self, i: i32, j: i32) -> bool {
        let cells_alive = self.get_neighbours(i, j);

        cells_alive == 3 || (cells_alive == 2 && self.cells[i as usize][j as usize].is_alive())
    }
}
