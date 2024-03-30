extern crate sdl2;

mod chip8;
mod opcodes;
mod stack;
mod utilities;

use sdl2::{event::Event, pixels::PixelFormatEnum};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use clap::Parser;

use std::time::{Duration, Instant};
use std::path::PathBuf;

use utilities::{SquareWave, DESIRED_AUDIO_SPEC};
use chip8::Chip8;

pub const SCALE : u32 = 16;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    program_file: PathBuf,

    #[arg(short)]
    scale: Option<u8>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // The emulator core
    // here load it with the parsed argument - game + scale
    let mut emulator = Chip8::new(cli.program_file)?;

    if let Some(scale) = cli.scale {
        run(&mut emulator, scale as u32)
    }
    else {
        run (&mut emulator, SCALE)
    }
}

fn run(emulator: &mut Chip8, scale: u32) -> Result<()> {
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
        chip8::SCREEN_WIDTH as u32 * scale,
        chip8::SCREEN_HEIGTH as u32 * scale,
        )
        .position_centered()
        .build()
        .unwrap();
 
    // Graphics related things
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();
    let mut tex_display = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGB24,
            chip8::SCREEN_WIDTH as u32,
            chip8::SCREEN_HEIGTH as u32,
        )
        .map_err(|e| e.to_string())?;

    // For getting the keyboard events...
    let mut event_pump = sdl_context.event_pump().unwrap();

    let frame_duration = Duration::new(0, 1_000_000_000u32 / 60);
    let mut timestamp = Instant::now();

    let mut key = 0u16;

    'running: loop {
        // Key handling
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    key |= match keycode {
                        Keycode::Num1 => 1 << 0x1,
                        Keycode::Num2 => 1 << 0x2,
                        Keycode::Num3 => 1 << 0x3,
                        Keycode::Num4 => 1 << 0xC,
                        Keycode::Q => 1 << 0x4,
                        Keycode::W => 1 << 0x5,
                        Keycode::E => 1 << 0x6,
                        Keycode::R => 1 << 0xD,
                        Keycode::A => 1 << 0x7,
                        Keycode::S => 1 << 0x8,
                        Keycode::D => 1 << 0x9,
                        Keycode::F => 1 << 0xE,
                        Keycode::Z => 1 << 0xA,
                        Keycode::X => 1 << 0x0,
                        Keycode::C => 1 << 0xB,
                        Keycode::V => 1 << 0xF,
                        _ => 0,
                    };
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    key &= !match keycode {
                        Keycode::Num1 => 1 << 0x1,
                        Keycode::Num2 => 1 << 0x2,
                        Keycode::Num3 => 1 << 0x3,
                        Keycode::Num4 => 1 << 0xC,
                        Keycode::Q => 1 << 0x4,
                        Keycode::W => 1 << 0x5,
                        Keycode::E => 1 << 0x6,
                        Keycode::R => 1 << 0xD,
                        Keycode::A => 1 << 0x7,
                        Keycode::S => 1 << 0x8,
                        Keycode::D => 1 << 0x9,
                        Keycode::F => 1 << 0xE,
                        Keycode::Z => 1 << 0xA,
                        Keycode::X => 1 << 0x0,
                        Keycode::C => 1 << 0xB,
                        Keycode::V => 1 << 0xF,
                        _ => 0,
                    };
                }
                _ => {}
            }
        }

        // Pass it to our emulator and execute opcode
        emulator.cycle(key)?;

        // Audio
        if emulator.tone() {
            audio.resume()
        }
        else {
            audio.pause();
        }

        // Draw graphics
        tex_display.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..chip8::SCREEN_HEIGTH {
                for x in 0..chip8::SCREEN_WIDTH / 8 {
                    let byte = emulator.gfx()[y * chip8::SCREEN_WIDTH / 8 + x];
                    for i in 0..8 {
                        let offset = y * pitch + (x * 8 + i) * 3;
                        let on = if byte & 1 << (7 - i) != 0 {
                            true
                        } else {
                            false
                        };
                        const FACTOR: u8 = 30;
                        let v = if on {
                            255
                        } else {
                            buffer[offset].saturating_sub(FACTOR)
                        };
                        buffer[offset] = v;
                        buffer[offset + 1] = v;
                        buffer[offset + 2] = v;
                    }
                }
            }
        })?;

        canvas.clear();
        canvas.copy(&tex_display, None, None)?;
        canvas.present();

        // FPS
        let now = Instant::now();
        let sleep_dur = frame_duration
            .checked_sub(now.saturating_duration_since(timestamp))
            .unwrap_or(Duration::new(0, 0));
        ::std::thread::sleep(sleep_dur);
        timestamp = now;
    }

    Ok(())
}
