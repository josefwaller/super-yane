// Simple application used for profiling
extern crate sdl3;

use core::ops::DerefMut;
use std::{
    env::args,
    fs,
    time::{Duration, Instant},
};

use log::debug;
use sdl3::{
    event::Event,
    keyboard::{KeyboardState, Keycode, Scancode},
    pixels::{PixelFormat, PixelFormatEnum},
    rect::Rect,
    render::{ScaleMode, SurfaceCanvas},
    surface::Surface,
};
use super_yane::{
    Console, InputPort, MASTER_CLOCK_SPEED_HZ,
    ppu::{PIXELS_PER_SCANLINE, SCANLINES},
    utils::color_to_rgb,
};

const SCREEN_SCALE: f32 = 3.0;
fn main() {
    let context = sdl3::init().expect("Unable to initialize SDL3: ");
    let video = context.video().expect("Unable to initialize video: ");
    let mut window = video
        .window(
            "Super Y.A.N.E.",
            256 * SCREEN_SCALE as u32,
            240 * SCREEN_SCALE as u32,
        )
        .build()
        .expect("Unable to build window:");

    window.raise();

    let mut console = match args().nth(1) {
        Some(s) => {
            let contents = fs::read(&s).expect(format!("Unable to read file '{}': ", s).as_str());
            if s.ends_with("sy.bin") {
                println!("Loading savestate");
                let mut c: Console = serde_brief::from_slice(&contents).unwrap();
                c.ppu_mut().reset_vram_cache();
                println!("Done loading!");
                c
            } else {
                println!("Loading ROM");
                Console::with_cartridge(&contents)
            }
        }
        None => panic!("No file provided"),
    };
    let mut event_pump = context
        .event_pump()
        .expect("Unable to initialize EventPump");
    let mut last_time = Instant::now();
    let mut actual_time = Duration::ZERO;
    let mut is_paused = false;
    let start_clocks = *console.total_master_clocks();
    'main_loop: loop {
        for e in event_pump.poll_iter() {
            match e {
                Event::Quit { .. } => break 'main_loop,
                Event::KeyDown { keycode, .. } => {
                    if keycode == Some(Keycode::P) {
                        is_paused = !is_paused;
                    } else if keycode == Some(Keycode::T) {
                        // Save savestate
                        let data: Vec<u8> = serde_brief::to_vec(&console).unwrap();
                        let file_name = "./savestate.sy.bin";
                        std::fs::write(file_name, data).unwrap();
                    }
                }
                _ => {}
            }
        }

        let keys = KeyboardState::new(&event_pump);
        let controller: InputPort = InputPort::StandardController {
            a: keys.is_scancode_pressed(Scancode::B),
            b: keys.is_scancode_pressed(Scancode::Space),
            x: keys.is_scancode_pressed(Scancode::N),
            y: keys.is_scancode_pressed(Scancode::M),
            left: keys.is_scancode_pressed(Scancode::A),
            right: keys.is_scancode_pressed(Scancode::D),
            up: keys.is_scancode_pressed(Scancode::W),
            down: keys.is_scancode_pressed(Scancode::S),
            start: keys.is_scancode_pressed(Scancode::R),
            select: keys.is_scancode_pressed(Scancode::F),
            r: keys.is_scancode_pressed(Scancode::E),
            l: keys.is_scancode_pressed(Scancode::Q),
        };
        console.input_ports_mut()[0] = controller;
        // Advance console
        if !is_paused {
            while !console.ppu().is_in_vblank() {
                console.advance_instructions(1);
            }
            while console.ppu().is_in_vblank() {
                console.advance_instructions(1);
            }
        }
        // Gather pixel data
        let mut pixel_data: [[u8; 4]; 256 * 240] = console
            .ppu()
            .screen_data_rgb()
            // SDL defaults to BGR
            .map(|[r, g, b]| [b, g, r, 255]);
        // Create surface from data
        let format = unsafe { PixelFormat::from_ll(PixelFormatEnum::ARGB8888.to_ll()) };
        let small_surface =
            Surface::from_data(pixel_data.as_flattened_mut(), 256, 240, 256 * 4, format).unwrap();
        // Get window surface
        let mut window_surface = window
            .surface(&event_pump)
            .expect("Unable to initialize surface: ");
        // Apply to window
        small_surface
            .blit_scaled(
                Rect::new(0, 0, 256, 240),
                window_surface.deref_mut(),
                Rect::new(
                    0,
                    0,
                    (SCREEN_SCALE * 256.0) as u32,
                    (SCREEN_SCALE * 240.0) as u32,
                ),
                ScaleMode::Nearest.into(),
            )
            .unwrap();
        // Refresh
        window_surface.finish().expect("Error while rendering: ");
        // Wait
        let emu_time = Duration::from_micros(
            1_000_000 * (console.total_master_clocks() - start_clocks) / MASTER_CLOCK_SPEED_HZ,
        );
        let now_time = Instant::now();
        if !is_paused {
            actual_time += now_time - last_time;
        }
        last_time = now_time;
        if emu_time > actual_time {
            std::thread::sleep(emu_time - actual_time);
        }
    }
}
