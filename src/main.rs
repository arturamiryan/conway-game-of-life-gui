#![deny(clippy::all)]
#![forbid(unsafe_code)]

use crate::gui::Framework;
use egui::Color32;
use pixels::{Error, Pixels, SurfaceTexture};
use std::{thread, time};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

mod cells;
mod gui;

use cells::*;

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

    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?;
        let framework = Framework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            SCALE as f32,
            &pixels,
        );
        (pixels, framework)
    };
    let mut grid = Grid::new(WIDTH as u32, HEIGHT as u32, [0, 0xff, 0, 0xf0]);
    grid.randomize();
    let mut stop = false;
    let mut speed = time::Duration::from_millis(0);

    event_loop.run(move |event, _, control_flow| {
        // Watch for keypresses
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
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

            window.request_redraw();
        }
        // Window redraw
        match event {
            Event::WindowEvent { event, .. } => {
                // Update egui inputs
                framework.handle_event(&event);
            }
            Event::RedrawEventsCleared => {
                // Prepare egui
                framework.prepare(&window);
                grid.draw(pixels.frame_mut());
                if let Err(err) = pixels.render_with(|encoder, render_target, context| {
                    // Render the world texture
                    context.scaling_renderer.render(encoder, render_target);

                    // Render egui
                    framework.render(encoder, render_target, context);

                    Ok(())
                }) {
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
    });
}
