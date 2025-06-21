use log::*;
use sdl2::{event::Event, sys::Window};
use simplelog::{Config, SimpleLogger};
use super_yane::Console;

fn main() {
    let context = sdl2::init().unwrap();
    let video = context.video().unwrap();
    let window = video.window("Super Y.A.N.E.", 256, 240).build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = context.event_pump().unwrap();

    SimpleLogger::init(log::LevelFilter::Debug, Config::default()).unwrap();
    info!("Logger initialized");

    let mut console = Console::with_cartridge(include_bytes!("../roms/cputest-basic.sfc"));

    let mut should_loop = true;
    while should_loop {
        canvas.clear();
        event_pump.poll_iter().for_each(|e| match e {
            Event::Quit { .. } => should_loop = false,
            _ => {}
        });

        console.advance_instructions(3);

        canvas.present();
    }
}
