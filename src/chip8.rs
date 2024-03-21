use std::{error::Error, fmt, fs::OpenOptions, io::Read, path::PathBuf};

use rand::random;

use crate::opcodes::Opcodes;
use crate::stack::{Stack, StackError};
use crate::utilities::{get_registers, get_register_and_value};

pub const MEMORY_SIZE: usize = 4086;
pub const GFX_SIZE: usize = 2048;
pub const STACK_SIZE: usize = 16;
pub const KEYS_SIZE: usize = 16;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGTH: usize = 32;

pub const REGISTERS_COUNT: usize = 16;
pub const CARY_REGISTER_IDX: usize = 0xF;

pub const LOADING_POINT: usize = 0x200;

#[derive(Debug, Clone)]
pub struct Chip8 {
    // Whole memory of the CHIP-8
    memory: [u8; MEMORY_SIZE],

    // general purpose reigsters V0,V1,..,VE;
    // VF - reserved for instructions
    registers: [u8; REGISTERS_COUNT],

    // address register
    I: u16,

    // program_counter (currently executing address)
    pc: u16,

    // program stack used to return when subroutine execute is called
    stack: Stack<STACK_SIZE>,

    // timers counting in 60Hz refresh rate
    delay_timer: u8,
    sound_timer: u8,

    // the graphic screen
    gfx: [u8; GFX_SIZE],

    // flag to update the graphics
    graphics: bool,

    // Currently pressed/released key
    key: u16,

    // Current keys state (0x1 - 0xF)
    keys: [char; KEYS_SIZE],

    key_reg_idx: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LoadInMemoryError;
impl Error for LoadInMemoryError { }

impl fmt::Display for LoadInMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error while loading program in memory!")
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
            I: 0,
            pc: 0x200,
            stack: Stack::<STACK_SIZE>::new(),
            delay_timer: 0,
            sound_timer: 0,
            gfx: [0; GFX_SIZE],
            graphics: true,
            key: 0,
            keys: ['\0'; KEYS_SIZE],
            key_reg_idx: REGISTERS_COUNT + 1,
        }
    }
}

impl Chip8 {
    pub fn new(program: PathBuf) -> Result<Self, LoadInMemoryError>  {
        let mut emulation = Chip8{..Default::default()};

        emulation.load_program_in_memory(program)?;

        Ok(emulation)
    }

    pub fn cycle(&mut self, key_pressed: u16) -> Result<(), Box<dyn std::error::Error>> {
        self.key = key_pressed;

        // get/fetch instruction
        let instruction_bytes = self.get_instruction_bytes();

        // decode operation code of instruction
        let instruction = Opcodes::try_from(instruction_bytes)?;

        // execute instruction
        let advance_pc = self.execute_instruction(instruction, instruction_bytes)?;
        if advance_pc {
            self.pc += 2;
        }

        // update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // TODO: Run SOUND - external crate?
                todo!();
            }
            self.sound_timer -= 1;
        }

        Ok(())
    }

    pub fn get_instruction_bytes(&self) -> u16 {
            ((self.memory[self.pc as usize] as u16) << 8)
            | self.memory[self.pc as usize + 1] as u16
    }

    pub fn draw_graphics(&mut self) -> bool {
        // This flag is being set to true on the following OpCodes (not on every cycle)
        // 0x00E0 - Clear Screen
        // 0xDXYN - Draw a sprite
        let current_flag = self.graphics;
        self.graphics = false;
        current_flag
    }

    pub fn handle_key(&mut self, key: u8) {
        // TODO: WIP
        self.keys [key as usize] = key as char;
    }

    fn load_program_in_memory (&mut self, program: PathBuf) -> Result<(), LoadInMemoryError> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(program)
            .expect("File not found");

        let mut program_bytes : Vec<u8> = Vec::new();
        file.read_to_end(&mut program_bytes).expect("Error in reading into vector");

        self.memory[LOADING_POINT..program_bytes.len() + LOADING_POINT].clone_from_slice(&program_bytes);

        Ok(())
    }

    // Returns bool flag if the PC shall be incremented or no + any errors occured
    fn execute_instruction(&mut self, instruction: Opcodes, instruction_bytes: u16) -> Result<bool, InstructionExecutionError> {
        match instruction {
            Opcodes::SysExecute => return Ok(true),

            Opcodes::ClearScreen => {
                self.graphics = true;
                return Ok(true);
            }

            Opcodes::Return => {
                if let Some(saved_pc) = self.stack.top() {
                    self.pc = saved_pc;
                    let _ = self.stack.pop()?;
                    return Ok(false);
                }
                else {
                    return Err(InstructionExecutionError("Stack error"));
                }
            }

            Opcodes::JumpTo => {
                self.pc = instruction_bytes & 0x0FFF;
                return Ok(false);
            }

            Opcodes::SubRoutineExecute => {
                let _ = self.stack.push(self.pc)?;
                self.pc = instruction_bytes & 0x0FFF;
                return Ok(false);
            }

            Opcodes::SkipIfEqualVal => {
                let (register_idx, value) = get_register_and_value(instruction_bytes)?;

                if self.registers[register_idx] == value {
                    self.pc += 4;
                    return Ok(false);
                }
                else {
                    return Ok(true);
                }
            }

            Opcodes::SkipIfNotEqualVal => {
                let (register_idx, value) = get_register_and_value(instruction_bytes)?;

                if self.registers[register_idx] == value {
                    self.pc += 4;
                    return Ok(false);
                }
                else {
                    return Ok(true);
                }
            }

            Opcodes::SkipIfEqualReg => {
                let (register_x_idx, register_y_idx) = get_registers(instruction_bytes)?;

                if self.registers[register_x_idx] == self.registers[register_y_idx] {
                    self.pc += 4;
                    return Ok(false);
                }
                else {
                    return Ok(true);
                }
            }

            Opcodes::StoreValInReg => {
                let (register_idx, value) = get_register_and_value(instruction_bytes)?;

                self.registers[register_idx] = value;
                return Ok(true);
            }

            Opcodes::AddValToReg => {
                let (register_idx, value) = get_register_and_value(instruction_bytes)?;

                self.registers[register_idx] += value;
                return Ok(true);
            }

            Opcodes::StoreRegInReg => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                self.registers[reg_x_idx] = self.registers[reg_y_idx];
                return Ok(true);
            }

            Opcodes::ORReg => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                self.registers[reg_x_idx] |= self.registers[reg_y_idx];
                return Ok(true);
            }

            Opcodes::ANDReg => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                self.registers[reg_x_idx] &= self.registers[reg_y_idx];
                return Ok(true);
            }

            Opcodes::XORReg => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                self.registers[reg_x_idx] ^= self.registers[reg_y_idx];
                return Ok(true);
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
                return Ok(true);
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
                return Ok(true);
            }

            Opcodes::StoreRegInRegShiftRight => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                self.registers[CARY_REGISTER_IDX] = self.registers[reg_y_idx] & 0x1;
                // !!! TODO:Check if we care about VY actually!!!
                self.registers[reg_x_idx] = self.registers[reg_y_idx] >> 1;

                return Ok(true);
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
                return Ok(true);
            }

            Opcodes::StoreRegInRegShiftLeft => {
                let (reg_x_idx, reg_y_idx) = get_registers(instruction_bytes)?;

                self.registers[CARY_REGISTER_IDX] = self.registers[reg_y_idx] & 0x10;
                // !!! TODO:Check if we care about VY actually!!!
                self.registers[reg_x_idx] = self.registers[reg_y_idx] << 1;

                return Ok(true);
            }
            
            Opcodes::SkipIfNotEqualReg => {
                let (register_x_idx, register_y_idx) = get_registers(instruction_bytes)?;

                if self.registers[register_x_idx] != self.registers[register_y_idx] {
                    self.pc += 4;
                    return Ok(false);
                }
                else {
                    return Ok(true);
                }
            }

            Opcodes::StoreMemoryInAddr => {
                let val = instruction_bytes & 0x0FFF;
                self.I = val;
                return Ok(true);
            }

            Opcodes::JumpToAddr => {
                let val = instruction_bytes & 0x0FFF;
                self.pc = val + self.registers[0] as u16;
                return Ok(false);
            }
            
            Opcodes::SetRandomNum => {
                let (reg_x, value) = get_register_and_value(instruction_bytes)?;
                let x = random::<u8>();
                self.registers[reg_x] = x & value;

                return Ok(true);
            }

            Opcodes::DrawSprite => {
                let (x_reg, y_reg) = get_registers(instruction_bytes)?;
                let height : usize = (instruction_bytes & 0x000F) as usize;
                let mut pixel : u8;

                self.registers[CARY_REGISTER_IDX] = 0;

                // TODO: Think of the Rust idiomatic way of doing it
                for y_line in 0usize..height {
                    pixel = self.memory[self.I as usize + y_line];

                    for x_line in 1usize..8 {
                        if (pixel & (0x80 >> x_line)) != 0 {
                            if self.gfx[x_reg + x_line + (y_reg + y_line) * 64] == 1 {
                                self.registers[CARY_REGISTER_IDX] = 1;
                            }

                            // We have our 2-dimensinal array of gfx set into
                            // 1 dimension 64 * 32 = 2048, so we need to get the
                            // y line cordinate (row) by multipling with 64
                            self.gfx[x_reg + x_line + (y_reg + y_line) * 64] ^= 1;
                        }
                    }
                }

                self.graphics = true;
                return Ok(true);
            }

            Opcodes::SkipIfPressed => {
                let (register_idx, _) = get_register_and_value(instruction_bytes)?;
                let key_val = self.registers[register_idx];

                if self.keys[key_val as usize] != '\0' {
                    self.pc += 4;
                    return Ok(false);
                }

                return Ok(true);
            }

            Opcodes::SkipIfNotPressed => {
                let (register_idx, _) = get_register_and_value(instruction_bytes)?;
                let key_val = self.registers[register_idx];

                if self.keys[key_val as usize] == '\0' {
                    self.pc += 4;
                    return Ok(false);
                }

                return Ok(true);
            }

            Opcodes::StoreDelayTimer => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;
                self.registers[reg_idx] = self.delay_timer;
                return Ok(true);
            }

            Opcodes::WaitKeypress => {
                // TODO: See how to block the program
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;
                self.key_reg_idx = reg_idx;
                return Ok(false);
            }

            Opcodes::SetDelayTimer => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;
                self.delay_timer = self.registers[reg_idx];
                return Ok(true);
            }

            Opcodes::SetSoundTimer => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;
                self.sound_timer = self.registers[reg_idx];
                return Ok(true);
            }

            Opcodes::AddValueToRegI => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;
                self.I += self.registers[reg_idx] as u16;
                return Ok(true);
            }

            Opcodes::SetIRegToStripeAddr => {
                // TODO
                return Ok(true);
            }

            Opcodes::StoreBCD => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;

                self.memory[(self.I) as usize] = self.registers[reg_idx] / 100;
                self.memory[(self.I + 1) as usize] = (self.registers[reg_idx] / 10) % 10;
                self.memory[(self.I + 2) as usize] = (self.registers[reg_idx] % 100) % 10;

                return Ok(true);
            }

            Opcodes::StoreRegsInMemoryFromRegI => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;

                for i in 0..reg_idx {
                    self.memory[self.I as usize + i] = self.registers[i];
                }

                return Ok(true);
            }

            Opcodes::LoadRegsInMemoryFromRegI => {
                let (reg_idx, _) = get_register_and_value(instruction_bytes)?;

                // TODO: Idiomatic way?
                for i in 0..reg_idx {
                    self.registers[i] = self.memory[self.I as usize + i]
                }
                
                return Ok(true);
           }

            _ => {
                dbg!(instruction);
                dbg!(instruction_bytes);
                todo!()
            }
        }
    }
}

fn get_key() -> u8 {
    todo!()
}


