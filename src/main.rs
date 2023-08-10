#![deny(clippy::all)]
#![forbid(unsafe_code)]

use pixels::{Error, Pixels, SurfaceTexture};
use std::{thread, time};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const SCALE: f64 = 2.0;
const WIDTH: i32 = (1920.0 / SCALE) as i32;
const HEIGHT: i32 = (1080.0 / SCALE) as i32;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scale = SCALE;
        let scaled_size = LogicalSize::new(WIDTH as f64 * scale, HEIGHT as f64 * scale);
        WindowBuilder::new()
            .with_title("Cellular Automato")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };
    let mut grid = Grid::new(WIDTH as u32, HEIGHT as u32);
    grid.randomize();
    let mut stop = false;
    let mut speed = time::Duration::from_millis(0);

    event_loop.run(move |event, _, control_flow| {
        // Window redraw
        match event {
            Event::RedrawEventsCleared => {
                grid.draw(pixels.frame_mut());
                if let Err(err) = pixels.render() {
                    eprintln!("{}", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                if !stop {
                    grid.check_rules();
                    thread::sleep(speed);
                }
            }
            _ => (),
        }

        // Watch for keypresses
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.key_pressed(VirtualKeyCode::S) || input.key_pressed(VirtualKeyCode::Space) {
                stop = !stop;
            }

            if input.key_pressed(VirtualKeyCode::R) {
                grid.randomize();
            }

            if input.key_pressed(VirtualKeyCode::B) {
                grid.set_blank();
            }

            if input.key_held(VirtualKeyCode::Plus) {
                if speed < time::Duration::from_secs(1) {
                    speed += time::Duration::from_millis(10);
                }
            }

            if input.key_held(VirtualKeyCode::Minus) {
                if speed > time::Duration::from_millis(0) {
                    speed -= time::Duration::from_millis(10);
                }
            }

            // Handle mouse.
            let (mouse_x, mouse_y) = input.mouse().unwrap_or_default();

            if input.mouse_pressed(0) {
                grid.set_pixel_alive(mouse_x / SCALE as f32, mouse_y / SCALE as f32);
            }

            if input.mouse_pressed(1) {
                grid.set_pixel_dead(mouse_x / SCALE as f32, mouse_y / SCALE as f32);
            }
        }
    });
}

#[derive(Clone, Copy)]
struct Cell {
    alive: bool,
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
}

struct Grid {
    cells: Vec<Vec<Cell>>,
    swap_cells: Vec<Vec<Cell>>,
}

impl Grid {
    pub fn new(width: u32, height: u32) -> Grid {
        Self {
            cells: vec![vec![Cell { alive: false }; width as usize]; height as usize],
            swap_cells: vec![vec![Cell { alive: false }; width as usize]; height as usize],
        }
    }

    pub fn randomize(&mut self) {
        for i in 0..HEIGHT as usize {
            for j in 0..WIDTH as usize {
                self.cells[i][j] = Cell {
                    alive: rand::random(),
                };
            }
        }
    }

    pub fn set_blank(&mut self) {
        for i in 0..HEIGHT as usize {
            for j in 0..WIDTH as usize {
                self.cells[i][j] = Cell { alive: false };
            }
        }
    }

    pub fn draw(&self, screen: &mut [u8]) {
        let mut pix = screen.chunks_exact_mut(4);
        for row in self.cells.iter() {
            for cell in row {
                let color = if cell.is_alive() {
                    [0, 0xff, 0, 0xf0]
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
                };
            }
        }
        std::mem::swap(&mut self.swap_cells, &mut self.cells);
    }

    fn set_pixel_alive(&mut self, x: f32, y: f32) {
        self.cells[y as usize][x as usize].set_alive();
    }

    fn set_pixel_dead(&mut self, x: f32, y: f32) {
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
