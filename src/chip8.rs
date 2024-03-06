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
        let opcode = self.get_opcode()?;

        // decode opcode or match it
        //  + execute opcode

        // update timer
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("TODO: Run SOUND");
            }
            self.sound_timer -= 1;
        }

        Ok(())
    }

    pub fn get_opcode(&self) -> Result<Opcodes, &str> {
        Opcodes::try_from
            (((self.memory[self.pc as usize] as u16) << 8)
            | self.memory[self.pc as usize + 1] as u16)
    }

    pub fn draw_graphics(&self) -> bool {
        // this flag is set not on every cycle rather on these 2 opcodes
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
}

