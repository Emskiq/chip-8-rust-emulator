use std::{error::Error, fmt, fs::OpenOptions, io::Read, path::PathBuf};

use crate::opcodes::Opcodes;

pub const MEMORY_SIZE: usize = 4086;
pub const GFX_SIZE: usize = 2048;
pub const STACK_SIZE: usize = 16;

pub const REGISTERS_COUNT: usize = 16;

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

    // stack pointer (top element)
    sp: u8,

    stack: [u16; STACK_SIZE],

    // timers counting in 60Hz refresh rate
    delay_timer: u8,
    sound_timer: u8,

    // the graphic screen
    gfx: [u8; GFX_SIZE],

    graphics: bool,
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
pub struct InstructionExecutionError;
impl Error for InstructionExecutionError { }

impl fmt::Display for InstructionExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error in executing instruction!")
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Chip8 {
            memory: [0; MEMORY_SIZE], // load fontset
            registers: [0; REGISTERS_COUNT],
            I: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; STACK_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            gfx: [0; GFX_SIZE],
            graphics: true,
        }
    }
}

impl Chip8 {
    pub fn new(program: PathBuf) -> Result<Self, LoadInMemoryError>  {
        let mut emulation = Chip8{..Default::default()};

        emulation.load_program_in_memory(program)?;

        Ok(emulation)
    }

    pub fn cycle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // get/fetch opcode
        let instruction_bytes = self.get_opcode_bytes();

        // decode opcode or match it
        //  + execute opcode
        let instruction = Opcodes::try_from(instruction_bytes)?;

        let advance_pc = self.execute_instruction(instruction, instruction_bytes)?;
        if advance_pc {
            self.pc += 2;
        }


        // update timer
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

    pub fn get_opcode_bytes(&self) -> u16 {
            ((self.memory[self.pc as usize] as u16) << 8)
            | self.memory[self.pc as usize + 1] as u16
    }

    pub fn draw_graphics(&self) -> bool {
        // This flag is being set to true on the following OpCodes (not on every cycle)
        // 0x00E0 - Clear Screen
        // 0xDXYN - Draw a sprite
        self.graphics
    }

    pub fn handle_key(&self) {
        todo!()
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

    fn execute_instruction(&mut self, instruction: Opcodes, instruction_bytes: u16) -> Result<bool, InstructionExecutionError> {
        match instruction {
            Opcodes::SysExecute => return Ok(true),
            Opcodes::ClearScreen => {
                self.graphics = true;
                return Ok(true);
            }
            Opcodes::Return => {
                self.pc = self.stack[(self.sp - 1) as usize];
                self.sp -= 1;
                return Ok(false);
            }
            Opcodes::SubRoutineExecute => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = instruction_bytes & 0x0FFF;
                return Ok(false);
            }

            _ => todo!(),
        }

        Err(InstructionExecutionError)
    }
}

