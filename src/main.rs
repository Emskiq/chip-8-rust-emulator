extern crate sdl2;
extern crate phf;

mod chip8;
mod opcodes;
mod stack;
mod utilities;

use phf::phf_map;

use sdl2::pixels::Color;

use std::fmt::{write, Debug};
use std::io::{Write, stdout, stdin};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use chip8::Chip8;

// TODO: Think whether we need separate class for the KEYBOARD Functionalities
static KEYS_MAP: phf::Map<char, u8> = phf_map! {
    '1' => 0x1,
    '2' => 0x2,
    '3' => 0x3,
    '4' => 0xc,
    'q' => 0x4,
    'w' => 0x5,
    'e' => 0x6,
    'r' => 0xD,
    'a' => 0x7,
    's' => 0x8,
    'd' => 0x9,
    'f' => 0xE,
    'z' => 0xA,
    'x' => 0x0,
    'c' => 0xB,
    'v' => 0xF,
};

pub const SCALE : u32 = 16;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

// Probably CLAP needed to parse the game file name and pass it to our emulator...
fn main() -> Result<()> {
    // The emulator core
    // here load it with the parsed argument - game + scale
    let mut emulator = Chip8::new("c8games/PONG".into())?;
    run(&mut emulator)
}

fn run(emulator: &mut Chip8) -> Result<()> {
    // Set up the Front-end of the emulator using SDL-2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;

    let window = video_subsystem.window("chip-8 emulator",
        chip8::SCREEN_WIDTH as u32 * SCALE, // TODO: Make the scale to be read from CLAP
        chip8::SCREEN_HEIGTH as u32 * SCALE,
        )
        .position_centered()
        .build()
        .unwrap();
 
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    // For getting the keyboard events...
    let mut event_pump = sdl_context.event_pump().unwrap();

    let frame_duration = Duration::new(0, 1_000_000_000u32 / 60);
    let mut timestamp = Instant::now();

    let key_pressed: u16 = 0x0;
    'running: loop {
        // Detect the key pressed - TODO...

        // Pass it to our emulator and execute opcode
        emulator.cycle(key_pressed)?; // maybe here add the key

        // Draw graphics
        /// Probably we will write graphics everytime
        /// - this flag will be used to update internaly the gfx array?
        // if emulator.draw_graphics() {
        //     draw_graphics();
        // }

        // FPS
        let now = Instant::now();
        let sleep_dur = frame_duration
            .checked_sub(now.saturating_duration_since(timestamp))
            .unwrap_or(Duration::new(0, 0));
        ::std::thread::sleep(sleep_dur);
        timestamp = now;
    }
}
