// Here define the opcodes as the list suggests with the enum case probably
// Got to look how to assign values to enums

pub enum Opcodes {
    ClearScreen = 0x00E0,
    SysExecute = 0x0000,
    Return = 0x00EE,
}

impl TryFrom<u16> for Opcodes {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0000 => Ok(Self::SysExecute),
            0x00E0 => Ok(Self::ClearScreen),
            0x00EE => Ok(Self::Return),
            _ => Err("Incorrect opcode!"),
        }
    }
}
