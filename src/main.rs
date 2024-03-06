mod chip8;
mod opcodes;

use chip8::Chip8;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    // Probably CLAP needed to parse the game file name and pass it to
    // our emulator...

    // let key_callback = Callbacks::new();

    let mut emulator = Chip8::new("c8games/PONG".into())?; // here load it with the parsed argument

    loop {
        emulator.cycle()?;

        if emulator.draw_graphics() {
            draw_graphics();
        }

        emulator.handle_key();
    }
}

fn draw_graphics() {
    todo!()
}
