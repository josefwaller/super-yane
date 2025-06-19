use sdl2::{event::Event, sys::Window};
use super_yane::Console;
fn main() {
    let mut console = Console::with_cartridge(include_bytes!("../roms/cputest-basic.sfc"));
    let context = sdl2::init().unwrap();
    let video = context.video().unwrap();
    let window = video.window("Super Y.A.N.E.", 256, 240).build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = context.event_pump().unwrap();

    let mut should_loop = true;
    while should_loop {
        canvas.clear();
        event_pump.poll_iter().for_each(|e| match e {
            Event::Quit { .. } => should_loop = false,
            _ => {}
        });

        canvas.present();
    }
}
