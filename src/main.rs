use std::env;
use std::thread;
use std::time::Duration;

mod display;
mod input;
mod processor;

use display::Display;
use input::Input;
use processor::Processor;
use processor::SuperChip;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filepath = &args[1];

    let sdl_context = match sdl2::init() {
        Ok(sdl_context) => sdl_context,
        Err(err) => panic!("SDL context could not initialize!  SDL_Error: {}", err),
    };

    let mut chippy = Processor::new(SuperChip);
    let mut display = Display::new(&sdl_context);
    let mut input = Input::new(&sdl_context);

    chippy.load(&filepath).unwrap();

    while let Ok(keypad) = input.poll() {
        chippy.set_keypad(&keypad);
        chippy.tick();

        if chippy.display_stale() {
            display.draw(chippy.get_screen());
        }

        // ensure 500Hz clock rate
        thread::sleep(Duration::from_millis(2));
    }
}
