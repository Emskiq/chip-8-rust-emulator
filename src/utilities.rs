use crate::chip8::{InstructionExecutionError, REGISTERS_COUNT};

use sdl2::audio::{AudioCallback, AudioSpecDesired};

////// Emulator utilities
pub fn get_registers(instruction_bytes: u16) -> Result<(usize, usize), InstructionExecutionError> {
    let idx_x = ((instruction_bytes >> 8) & 0x000F) as usize;
    let idx_y = ((instruction_bytes >> 4) & 0x000F) as usize;

    if idx_x >= REGISTERS_COUNT || idx_y >= REGISTERS_COUNT {
        Err(InstructionExecutionError("Registers indeces out of range!"))
    }
    else {
        Ok((idx_x, idx_y))
    }
}

pub fn get_register_and_value(instruction_bytes: u16) -> Result<(usize, u8), InstructionExecutionError> {
    let idx = ((instruction_bytes >> 8) & 0x000F) as usize;
    let val = (instruction_bytes & 0x00FF) as u8;

    if idx >= REGISTERS_COUNT {
        Err(InstructionExecutionError("Register idx out of range!"))
    }
    else {
        Ok((idx, val))
    }
}

////// Audio utilities
pub struct SquareWave {
    pub phase_inc: f32,
    pub phase: f32,
    pub volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub const DESIRED_AUDIO_SPEC: AudioSpecDesired = AudioSpecDesired {
    freq: Some(44100),
    channels: Some(1),  // mono
    samples: None       // default sample size
};
