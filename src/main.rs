extern crate sdl2;

mod chip8;
mod opcodes;
mod stack;
mod utilities;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use utilities::{SquareWave, DESIRED_AUDIO_SPEC};

use std::time::{Duration, Instant};

use chip8::Chip8;

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

    let audio = audio_subsystem.open_playback(None, &DESIRED_AUDIO_SPEC, |spec| {
        // initialize the audio callback
        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25,
        }
    })?;

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

    let mut key: u8 = 0;
    let mut is_pressed = false;

    'running: loop {
        canvas.clear();

        // Key handling
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |

                // Key press
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    is_pressed = true;
                    match keycode {
                        Keycode::Num1 => key = 0x1,
                        Keycode::Num2 => key = 0x2,
                        Keycode::Num3 => key = 0x3,
                        Keycode::Num4 => key = 0xC,
                        Keycode::Q    => key = 0x4,
                        Keycode::W    => key = 0x5,
                        Keycode::E    => key = 0x6,
                        Keycode::R    => key = 0xD,
                        Keycode::A    => key = 0x7,
                        Keycode::S    => key = 0x8,
                        Keycode::D    => key = 0x9,
                        Keycode::F    => key = 0xE,
                        Keycode::Z    => key = 0xA,
                        Keycode::X    => key = 0x0,
                        Keycode::C    => key = 0xB,
                        Keycode::V    => key = 0xF,
                        _             => key = 16,
                    }
                }

                // Key release
                Event::KeyUp { keycode: Some(Keycode::Escape), ..} => break 'running,
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    is_pressed = false;
                    match keycode {
                        Keycode::Num1 => key = 0x1,
                        Keycode::Num2 => key = 0x2,
                        Keycode::Num3 => key = 0x3,
                        Keycode::Num4 => key = 0xC,
                        Keycode::Q    => key = 0x4,
                        Keycode::W    => key = 0x5,
                        Keycode::E    => key = 0x6,
                        Keycode::R    => key = 0xD,
                        Keycode::A    => key = 0x7,
                        Keycode::S    => key = 0x8,
                        Keycode::D    => key = 0x9,
                        Keycode::F    => key = 0xE,
                        Keycode::Z    => key = 0xA,
                        Keycode::X    => key = 0x0,
                        Keycode::C    => key = 0xB,
                        Keycode::V    => key = 0xF,
                        _             => key = 16,
                    }
                }

                _ => {}
            }
        }

        // Pass it to our emulator and execute opcode
        emulator.cycle(key, is_pressed)?; // maybe here add the key

        if emulator.tone() {
            audio.resume()
        }
        else {
            audio.pause();
        }

        // Draw graphics
        /// Probably we will write graphics everytime
        /// - this flag will be used to update internaly the gfx array?
        // if emulator.draw_graphics() {
        //     draw_graphics();
        // }

        canvas.present();
        // FPS - first check with the basic one
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        // let now = Instant::now();
        // let sleep_dur = frame_duration
        //     .checked_sub(now.saturating_duration_since(timestamp))
        //     .unwrap_or(Duration::new(0, 0));
        // ::std::thread::sleep(sleep_dur);
        // timestamp = now;
    }

    Ok(())
}
