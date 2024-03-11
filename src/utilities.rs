use crate::chip8::{InstructionExecutionError, REGISTERS_COUNT};

pub fn get_registers(instruction_bytes: u16) -> Result<(usize, usize), InstructionExecutionError> {
    let idx_x = (instruction_bytes & 0x0F00 >> 8) as usize;
    let idx_y = (instruction_bytes & 0x00F0 >> 4) as usize;

    if idx_x >= REGISTERS_COUNT || idx_y >= REGISTERS_COUNT {
        Err(InstructionExecutionError("Registers indeces out of range!"))
    }
    else {
        Ok((idx_x, idx_y))
    }
}

pub fn get_register_and_value(instruction_bytes: u16) -> Result<(usize, u8), InstructionExecutionError> {
    let idx = (instruction_bytes & 0x0F00 >> 8) as usize;
    let val = (instruction_bytes & 0x00FF) as u8;

    if idx >= REGISTERS_COUNT {
        Err(InstructionExecutionError("Register idx out of range!"))
    }
    else {
        Ok((idx, val))
    }
}
