// Here define the opcodes as the list suggests with the enum case probably
// Got to look how to assign values to enums

#[derive(Debug)]
pub enum Opcodes {
    SysExecute = 0x0000,
    ClearScreen = 0x00E0,
    Return = 0x00EE,
    JumpTo = 0x1000,
    SubRoutineExecute = 0x2000,
    SkipIfEqualVal = 0x3000,
    SkipIfNotEqualVal = 0x4000,
    SkipIfEqualReg = 0x5000,
    StoreValInReg = 0x6000,
    AddValToReg = 0x7000,
    StoreRegInReg = 0x8000,
    ORReg = 0x8001,
    ANDReg = 0x8002,
    XORReg = 0x8003,
    AddRegToReg = 0x8004,
    SubRegFromReg = 0x8005,
    StoreRegInRegShiftRight = 0x8006,
    SetRegMinusReg = 0x8007,
    StoreRegInRegShiftLeft = 0x800E,
    SkipIfNotEqualReg = 0x9000,
    StoreMemoryInAddr = 0xA000,
    JumpToAddr = 0xB000,
    SetRandomNum = 0xC000,
    DrawSprite = 0xD000,
    SkipIfPressed = 0xE09E,
    SkipIfNotPressed = 0xE0A1,
    StoreDelayTimer = 0xF007,
    WaitKeypress = 0xF00A,
    SetDelayTimer = 0xF015,
    SetSoundTimer = 0xF018,
    AddValueToRegI = 0xF01E,
    SetIReg = 0xF029,
    StoreBCD = 0xF033,
    StoreRegsInMemoryFromRegI = 0xF055,
    FillRegsInMemoryFromRegI = 0xF065,
}

impl TryFrom<u16> for Opcodes {
    type Error = &'static str;

    // Try converting 2 bytes instruction (represented as u16)
    // to the corresponing Operation Code
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value & 0xF000 {
            0x0000 =>
                match value & 0x0FFF {
                    0x0000 => Ok(Self::SysExecute),
                    0x00E0 => Ok(Self::ClearScreen),
                    0x00EE => Ok(Self::Return),
                    _ => Err("Incorrect opcode"),
                },
            0x1000 => Ok(Self::JumpTo),
            0x2000 => Ok(Self::SubRoutineExecute),
            0x3000 => Ok(Self::SkipIfEqualVal),
            0x4000 => Ok(Self::SkipIfNotEqualVal),
            0x5000 => Ok(Self::SkipIfEqualReg),
            0x6000 => Ok(Self::StoreValInReg),
            0x7000 => Ok(Self::AddValToReg),
            0x8000 =>
                match value & 0xF00F {
                    0x8000 => Ok(Self::StoreRegInReg),
                    0x8001 => Ok(Self::ORReg),
                    0x8002 => Ok(Self::ANDReg),
                    0x8003 => Ok(Self::XORReg),
                    0x8004 => Ok(Self::AddRegToReg),
                    0x8005 => Ok(Self::SubRegFromReg),
                    0x8006 => Ok(Self::StoreRegInRegShiftRight),
                    0x8007 => Ok(Self::SetRegMinusReg),
                    0x800E => Ok(Self::StoreRegInRegShiftLeft),
                    _ => Err("Incorrect opcode"),
                },
            0x9000 => Ok(Self::SkipIfNotEqualReg),
            0xA000 => Ok(Self::StoreMemoryInAddr),
            0xB000 => Ok(Self::JumpToAddr),
            0xC000 => Ok(Self::SetRandomNum),
            0xD000 => Ok(Self::DrawSprite),
            0xE000 =>
                match value & 0xF0FF {
                    0xE09E => Ok(Self::SkipIfPressed),
                    0xE0A1 => Ok(Self::SkipIfNotPressed),
                    _ => Err("Incorrect opcode"),
                },
            0xF000 =>
                match value & 0xF0FF {
                    0xF007 => Ok(Self::StoreDelayTimer),
                    0xF00A => Ok(Self::WaitKeypress),
                    0xF015 => Ok(Self::SetDelayTimer),
                    0xF017 => Ok(Self::SetSoundTimer),
                    0xF01E => Ok(Self::AddValueToRegI),
                    0xF029 => Ok(Self::SetIReg),
                    0xF033 => Ok(Self::StoreBCD),
                    0xF055 => Ok(Self::StoreRegsInMemoryFromRegI),
                    0xF065 => Ok(Self::FillRegsInMemoryFromRegI),
                    _ => Err("Incorrect opcode"),
                },

            _ => Err("Incorrect opcode!"),
        }
    }
}
