use std::{isize, usize};
use std::{error::Error, fmt, fs::OpenOptions, io::Read, path::PathBuf};

use rand::random;
use log::debug;

use crate::opcodes::Opcodes;
use crate::stack::{Stack, StackError};
use crate::utilities::{get_registers, get_register_and_value};

const SPRITE_CHARS: [[u8; 5]; 0x10] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
    [0x20, 0x60, 0x20, 0x20, 0x70], // 1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
    [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
    [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
    [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
    [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
    [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
];
const SPRITE_CHARS_ADDR: u16 = 0x0000;

pub const MEMORY_SIZE: usize = 4086;
pub const STACK_SIZE: usize = 16;
pub const KEYS_SIZE: usize = 17; // If we press invalid key

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGTH: usize = 32;

pub const REGISTERS_COUNT: usize = 16;
pub const CARY_REGISTER_IDX: usize = 0xF;

pub const LOADING_POINT: usize = 0x200;

pub const FRAME_TIME: isize = 16666; // this is in microseconds

#[derive(Debug, Clone)]
pub struct Chip8 {
    // Whole memory of the CHIP-8
    memory: [u8; MEMORY_SIZE],

    // general purpose reigsters V0,V1,..,VE;
    // VF - reserved for instructions
    registers: [u8; REGISTERS_COUNT],

    // address register
    i: u16,

    // program_counter (currently executing address)
    pc: u16,

    // program stack used to return when subroutine execute is called
    stack: Stack<STACK_SIZE>,

    // timers counting in 60Hz refresh rate
    delay_timer: u8,

    sound_timer: u8,
    run_sound: bool,

    // the graphic screen
    gfx: [u8; SCREEN_WIDTH * SCREEN_HEIGTH / 8],

    // Current keys state (0x1 - 0xF)
    keys: [bool; KEYS_SIZE],

    // time in seconds for executing operation
    time: isize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LoadInMemoryError(&'static str);
impl Error for LoadInMemoryError { }

impl fmt::Display for LoadInMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error while loading program in memory! Error: {}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PcOutOfMemoryBounds(u16);
impl Error for PcOutOfMemoryBounds { }

impl fmt::Display for PcOutOfMemoryBounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Program counter {} is out of memory bounds!", self.0)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InstructionExecutionError(pub &'static str);
impl Error for InstructionExecutionError { }

impl fmt::Display for InstructionExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error in executing instruction: {} ", self.0)
    }
}

impl From<StackError> for InstructionExecutionError {
    fn from(value: StackError) -> Self {
        InstructionExecutionError(value.0)
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Chip8 {
            memory: [0; MEMORY_SIZE], // load fontset
            registers: [0; REGISTERS_COUNT],
            i: 0,
            pc: 0x200,
            stack: Stack::<STACK_SIZE>::new(),
            delay_timer: 0,
            sound_timer: 0,
            run_sound: false,
            gfx: [0; SCREEN_WIDTH * SCREEN_HEIGTH / 8],
            keys: [false; KEYS_SIZE],
            time: 0,
        }
    }
}

impl Chip8 {
    pub fn new(program: PathBuf) -> Result<Self, LoadInMemoryError>  {
        let mut emulation = Chip8{..Default::default()};

        emulation.load_font_set_in_memory();
        emulation.load_program_in_memory(program)?;

        Ok(emulation)
    }

    pub fn cycle(&mut self, key: u8, is_pressed: bool) -> Result<(), Box<dyn std::error::Error>> {
        // store pressed key
        self.handle_key(key, is_pressed);

        // update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                self.run_sound = true;
            }
            self.sound_timer -= 1;
        }

        // --- Execution of an instruction in a FRAME
        self.time += FRAME_TIME;
        while self.time > 0 {
            if self.pc as usize > MEMORY_SIZE - 1 {
                return Err(PcOutOfMemoryBounds(self.pc).into());
            }

            // get/fetch instruction
            let instruction_bytes = self.get_instruction_bytes();
            // debug!("intstruction bytes: {:#06x}", instruction_bytes);

            // decode operation code of instruction
            let instruction = Opcodes::try_from(instruction_bytes)?;
            // debug!("intstruction bytes: {:#06x} -> {}", instruction_bytes, instruction);

            // execute instruction + get overtime that it takes to be executed originally
            let overtime = self.execute_instruction(instruction, instruction_bytes)?;

            self.time -= overtime as isize;
        }


        Ok(())
    }

    fn get_instruction_bytes(&self) -> u16 {
            ((self.memory[self.pc as usize] as u16) << 8)
            | self.memory[self.pc as usize + 1] as u16
    }

    pub fn tone(&mut self) -> bool {
        let current_flag = self.run_sound;
        self.run_sound = false;
        current_flag
    }

    pub fn handle_key(&mut self, key: u8, is_pressed: bool) {
        if is_pressed {
            self.keys [key as usize] = true;
        }
        else {
            self.keys [key as usize] = false;
        }
    }

    fn load_program_in_memory (&mut self, program: PathBuf) -> Result<(), LoadInMemoryError> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(program)
            .expect("File not found");

        let mut program_bytes : Vec<u8> = Vec::new();
        file.read_to_end(&mut program_bytes).expect("Error in reading into vector");

        if program_bytes.len() > MEMORY_SIZE - LOADING_POINT {
            return Err(LoadInMemoryError("Program is larger!"));
        }

        self.memory[LOADING_POINT..program_bytes.len() + LOADING_POINT].clone_from_slice(&program_bytes);

        Ok(())
    }

    // Returns bool flag if the PC shall be incremented or no + any errors occured
    // TODO: Return actually only overtime w/out advance_pc - it is less coe if you add it here
    fn execute_instruction(&mut self, instruction: Opcodes, instruction_bytes: u16) -> Result<isize, InstructionExecutionError> {
        match instruction {
            Opcodes::SysExecute => return Ok(100),

            Opcodes::ClearScreen => {
                for b in self.gfx.iter_mut() {
                    *b = 0;
                }
                self.pc += 2;
                return Ok(109);
            }

            Opcodes::Return => {
                if let Some(saved_pc) = self.stack.top() {
                    self.pc = saved_pc;
                    let _ = self.stack.pop()?;
                    return Ok(105);
                }
                else {
                    return Err(InstructionExecutionError("Stack error"));
                }
            }

            Opcodes::JumpTo => {
                self.pc = instruction_bytes & 0x0FFF;
                return Ok(105);
            }

            Opcodes::SubRoutineExecute => {
                let _ = self.stack.push(self.pc + 2)?;
                self.pc = instruction_bytes & 0x0FFF;
                return Ok(105);
            }

            Opcodes::SkipIfEqualVal => {
                let (register_idx, value) = get_register_and_value(instruction_bytes)?;

                // debug!("Skip if reg: {} == val {}", self.registers[register_idx], value);

                if self.registers[register_idx] == value {
                    self.pc += 4;
                    return Ok(61);
                }
                else {
                    self.pc += 2;
                    return Ok(61);
                }
            }

            Opcodes::SkipIfNotEqualVal => {
                let (register_idx, value) = get_register_and_value(instruction_bytes)?;

                if self.registers[register_idx] != value {
                    self.pc += 4;
                    return Ok(61);
                }
                else {
                    self.pc += 2;
                    return Ok(61);
                }
            }

            Opcodes::SkipIfEqualReg => {
                let (register_x_idx, register_y_idx) = get_registers(instruction_bytes)?;

                if self.registers[register_x_idx] == self.registers[register_y_idx] {
                    self.pc += 4;
                    return Ok(61);
                }
                else {
                    self.pc += 2;
                    return Ok(61)
                }
            }

            Opcodes::StoreValInReg => {
                let (register_idx, value) = get_register_and_value(instruction_bytes)?;

                // debug!("Store Val {} in Reg IDX {}", value, register_idx);
                self.registers[register_idx] = value;
                self.pc += 2;
                return Ok(27);
            }

            Opcodes::AddValToReg => {
                let (register_idx, value) = get_register_and_value(instruction_bytes)?;

                self.registers[register_idx] = self.registers[register_idx].wrapping_add(value);
                self.pc += 2;
                return Ok(45);
            }

            Opcodes::StoreRegInReg => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                self.registers[reg_x_idx] = self.registers[reg_y_idx];

                self.pc += 2;
                return Ok(27);
            }

            Opcodes::ORReg => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                self.registers[reg_x_idx] |= self.registers[reg_y_idx];

                self.pc += 2;
                return Ok(200);
            }

            Opcodes::ANDReg => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                // debug!("AND REGistets {} &= {}", self.registers[reg_x_idx], self.registers[reg_y_idx]);
                self.registers[reg_x_idx] &= self.registers[reg_y_idx];

                self.pc += 2;
                return Ok(200);
            }

            Opcodes::XORReg => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                self.registers[reg_x_idx] ^= self.registers[reg_y_idx];

                self.pc += 2;
                return Ok(200);
            }

            Opcodes::AddRegToReg => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                let sum : u16 = self.registers[reg_x_idx] as u16 + self.registers[reg_y_idx] as u16;

                if sum > 0xFF {
                    self.registers[CARY_REGISTER_IDX] = 1; // carry
                }
                else {
                    self.registers[CARY_REGISTER_IDX] = 0;
                }

                self.registers[reg_x_idx] = sum as u8;
                
                self.pc += 2;
                return Ok(45);
            }

            Opcodes::SubRegFromReg => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                let sum : i16 = self.registers[reg_x_idx] as i16 - self.registers[reg_y_idx] as i16;

                if sum < 0 {
                    self.registers[CARY_REGISTER_IDX] = 1; // carry
                }
                else {
                    self.registers[CARY_REGISTER_IDX] = 0;
                }

                self.registers[reg_x_idx] = sum as u8;
                
                self.pc += 2;
                return Ok(200);
            }

            Opcodes::StoreRegInRegShiftRight => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                self.registers[CARY_REGISTER_IDX] = self.registers[reg_y_idx] & 0b00000001;
                self.registers[reg_x_idx] = self.registers[reg_y_idx] >> 1;

                self.pc += 2;
                return Ok(200);
            }

            Opcodes::SetRegMinusReg => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                if self.registers[reg_x_idx] > self.registers[reg_y_idx] {
                    self.registers[CARY_REGISTER_IDX] = 1;
                }
                else {
                    self.registers[CARY_REGISTER_IDX] = 0;
                }

                self.registers[reg_x_idx] = self.registers[reg_y_idx] - self.registers[reg_x_idx];

                self.pc += 2;
                return Ok(200);
            }

            Opcodes::StoreRegInRegShiftLeft => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                self.registers[CARY_REGISTER_IDX] = (self.registers[reg_x_idx] & 0b10000000) >> 7;
                self.registers[reg_x_idx] = self.registers[reg_y_idx] << 1;

                self.pc += 2;
                return Ok(200);
            }
            
            Opcodes::SkipIfNotEqualReg => {
                let (register_x_idx, register_y_idx) = get_registers(instruction_bytes)?;

                if self.registers[register_x_idx] != self.registers[register_y_idx] {
                    self.pc += 4;
                    return Ok(61);
                }
                else {
                    self.pc += 2;
                    return Ok(61);
                }
            }

            Opcodes::StoreMemoryInAddr => {
                let val = instruction_bytes & 0x0FFF;
                // debug!("Store val {} in memory I", val);
                self.i = val;
                self.pc += 2;
                return Ok(55);
            }

            Opcodes::JumpToAddr => {
                let val = instruction_bytes & 0x0FFF;
                self.pc = val + self.registers[0] as u16;
                return Ok(105);
            }
            
            Opcodes::SetRandomNum => {
                let (reg_x, value) = get_register_and_value(instruction_bytes)?;
                let x = random::<u8>();
                self.registers[reg_x] = x & value;

                self.pc += 2;
                return Ok(164);
            }

            Opcodes::DrawSprite => {
                let (x_reg, y_reg) = get_registers(instruction_bytes)?;
                let height : usize = (instruction_bytes & 0x000F) as usize;

                let pos_x = self.registers[x_reg] % 64;
                let pos_y = self.registers[y_reg] % 32;

                debug!("pos_x: {}, pos_y: {}", pos_x, pos_y);

                let gfx = &mut self.gfx;
                let shift = pos_x % 8;
                let col_a = pos_x as usize / 8;
                let col_b = (col_a + 1) % (SCREEN_WIDTH / 8);
                let mut collision = 0;
                for i in 0..(height as usize) {
                    let byte = self.memory[self.i as usize + i];
                    let y = (pos_y as usize + i) % SCREEN_HEIGTH;
                    let a = byte >> shift;
                    let fb_a = &mut gfx[y * SCREEN_WIDTH / 8 + col_a];
                    collision |= *fb_a & a;
                    *fb_a ^= a;
                    if shift != 0 {
                        let b = byte << (8 - shift);
                        let fb_b = &mut gfx[y * SCREEN_WIDTH / 8 + col_b];
                        collision |= *fb_b & b;
                        *fb_b ^= b;
                    }
                }
                self.registers[CARY_REGISTER_IDX] = if collision != 0 { 1 } else { 0 }; 

                self.pc += 2;
                return Ok(22734);
            }

            Opcodes::SkipIfPressed => {
                let (register_idx, _) = get_register_and_value(instruction_bytes)?;
                let key_val = self.registers[register_idx];

                debug!("Skip if pressed key {}", key_val);

                if self.keys[key_val as usize] == true {
                    self.pc += 4;
                    return Ok(73);
                }

                self.pc += 2;
                return Ok(73);
            }

            Opcodes::SkipIfNotPressed => {
                let (register_idx, _) = get_register_and_value(instruction_bytes)?;
                let key_val = self.registers[register_idx];

                debug!("Skip if NOT pressed key {}", key_val);


                if self.keys[key_val as usize] == false {
                    self.pc += 4;
                    return Ok(73);
                }

                self.pc += 2;
                return Ok(73);
            }

            Opcodes::StoreDelayTimer => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;
                self.registers[reg_idx] = self.delay_timer;
                self.pc += 2;
                return Ok(27);
            }

            Opcodes::WaitKeypress => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;

                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.registers[reg_idx] = i as u8;
                        self.pc += 2;
                        return Ok(200);
                    }
                }
                return Ok(200);
            }

            Opcodes::SetDelayTimer => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;
                self.delay_timer = self.registers[reg_idx];
                self.pc += 2;
                return Ok(45);
            }

            Opcodes::SetSoundTimer => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;
                self.sound_timer = self.registers[reg_idx];
                self.pc += 2;
                return Ok(45);
            }

            Opcodes::AddValueToRegI => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;
                self.i += self.registers[reg_idx] as u16;
                self.pc += 2;
                return Ok(86);
            }

            Opcodes::SetIRegToStripeAddr => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;
                self.i = SPRITE_CHARS_ADDR + self.registers[reg_idx] as u16 * 5;
                self.pc += 2;
                return Ok(91);
            }

            Opcodes::StoreBCD => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;
                let v = self.registers[reg_idx];
                let d2 = v / 100;
                let v = v - d2 * 100;
                let d1 = v / 10;
                let v = v - d1 * 10;
                let d0 = v / 1;
                
                debug!("storing bcds of REG IDX {} : {}, {}, {}", reg_idx, d2, d1, d0);

                self.memory[(self.i) as usize] = d2;
                self.memory[(self.i + 1) as usize] = d1;
                self.memory[(self.i + 2) as usize] = d0;

                self.pc += 2;
                return Ok(927);
            }

            Opcodes::StoreRegsInMemoryFromRegI => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;

                for i in 0..reg_idx + 1 {
                    self.memory[self.i as usize + i] = self.registers[i];
                }

                self.pc += 2;
                return Ok(605);
            }

            Opcodes::LoadRegsInMemoryFromRegI => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;

                for i in 0..reg_idx + 1 {
                    self.registers[i] = self.memory[self.i as usize + i]
                }
                
                self.pc += 2;
                return Ok(605);
           }
        }
    }

    pub fn gfx(&self) -> [u8; SCREEN_WIDTH * SCREEN_HEIGTH / 8] {
        self.gfx
    }

    fn load_font_set_in_memory(&mut self) {
        for (i, sprite) in SPRITE_CHARS.iter().enumerate() {
            let p = SPRITE_CHARS_ADDR as usize + i * sprite.len();
            self.memory[p..p + sprite.len()].copy_from_slice(sprite)
        }
    }
}
