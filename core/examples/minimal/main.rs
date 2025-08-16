// Simple application used for profiling
extern crate sdl3;

use std::{env::args, fs};

use sdl3::{event::Event, pixels::PixelFormatEnum};
use super_yane::{
    Console,
    ppu::{PIXELS_PER_SCANLINE, SCANLINES},
    utils::color_to_rgb,
};
fn main() {
    let context = sdl3::init().expect("Unable to initialize SDL3: ");
    let video = context.video().expect("Unable to initialize video: ");
    let window = video
        .window("Super Y.A.N.E.", 256 as u32, 240 as u32)
        .build()
        .expect("Unable to build window:");

    let cartridge_contents = match args().nth(1) {
        Some(s) => fs::read(&s).expect(format!("Unable to read file '{}': ", s).as_str()),
        None => panic!("No .SFC file provided"),
    };
    let mut console = Console::with_cartridge(&cartridge_contents);
    let mut event_pump = context
        .event_pump()
        .expect("Unable to initialize EventPump");
    'main_loop: loop {
        for e in event_pump.poll_iter() {
            match e {
                Event::Quit { .. } => break 'main_loop,
                _ => {}
            }
        }
        // Advance console
        while !console.ppu().is_in_vblank() {
            console.advance_instructions(1);
        }
        while console.ppu().is_in_vblank() {
            console.advance_instructions(1);
        }
        // Gather pixel data
        let pixel_data: Vec<u8> = console
            .ppu()
            .screen_buffer
            .iter()
            .map(|s| color_to_rgba(*s))
            .flatten()
            .map(|s| s)
            .collect();
        // Apply to window
        let mut surface = window
            .surface(&event_pump)
            .expect("Unable to initialize surface: ");
        surface.with_lock_mut(|p| p.copy_from_slice(&pixel_data));
        surface.finish().expect("Error while rendering: ");
    }
}
fn color_to_rgba(c: u16) -> [u8; 4] {
    [
        ((c / 0x400) as u8 & 0x1F) << 3,
        ((c / 0x20) as u8 & 0x1F) << 3,
        (c as u8 & 0x1F) << 3,
        255,
    ]
}
