#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::{thread, time};

use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: i32 = 400;
const HEIGHT: i32 = 300;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64 * 3.0, HEIGHT as f64 * 3.0);
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
    let ten_millis = time::Duration::from_millis(10);
    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            grid.draw(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                eprintln!("{}", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            if input.key_pressed(VirtualKeyCode::R) {
                grid.randomize();
            }
            if input.key_pressed(VirtualKeyCode::S) {
                stop = !stop;
            }
        }

        if !stop {
            grid.check_rules();
        }
        window.request_redraw();
        thread::sleep(ten_millis);
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
}

struct Grid {
    cells: Vec<Vec<Cell>>,
    swap_cells: Vec<Vec<Cell>>,
}

impl Grid {
    pub fn new(width: u32, height: u32) -> Grid {
        Self {
            cells: vec![vec![Cell {alive: false}; width as usize]; height as usize],
            swap_cells: vec![vec![Cell {alive: false}; width as usize]; height as usize],
        }
    }
    pub fn randomize(&mut self) {
        for i in 0..HEIGHT as usize {
            for j in 0..WIDTH as usize {
                self.cells[i][j] = Cell { alive: rand::random()};
            }
        }
    }
    pub fn draw(&self, screen: &mut [u8]) {
        let mut pix = screen.chunks_exact_mut(4);
        for row in self.cells.iter() {
            for cell in row {
                let color = if cell.is_alive() {
                    [0, 0xff, 0xff, 0xff]
                } else {
                    [0, 0, 0xff, 0xff]
                };
                pix.next().unwrap().copy_from_slice(&color);
            }
        }
    }
    pub fn check_rules(&mut self) {
        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                self.cells[i as usize][j as usize] = Cell { alive: self.decide(i, j)};
            }
        }
        std::mem::swap(&mut self.swap_cells, &mut self.cells);
    }
    fn decide(&self, i: i32, j: i32) -> bool {
        let mut cells_alive = 0;
        for i1 in (i-1)..=(i+1) {
            for j1 in (j-1)..=(j+1) {
                if self.cells[((i1 + HEIGHT) % HEIGHT) as usize][((j1 + WIDTH) % WIDTH) as usize].is_alive() {
                    cells_alive += 1;
                }
            }
        }

        if self.cells[i as usize][ j as usize].is_alive() {
            cells_alive -= 1;
        }

        cells_alive == 3 || (cells_alive == 2 && self.cells[i as usize][ j as usize].is_alive())
    }
}
