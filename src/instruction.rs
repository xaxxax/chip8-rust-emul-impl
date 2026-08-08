// Debug allows for :? formatting
// Copy allows for copy of value implicitly, without needing to go against borrow checker
// Copy requires Clone, so it must also be passed

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    // display ISA
    ClearScreen,                                // 00E0
    Draw { x: usize, y: usize, height: usize }, // DXYN

    // flow for getting around memory
    Jump { address: u16 },           // 1NNN
    JumpWithOffset { address: u16 }, // BNNN
    Call { address: u16 },           // 2NNN
    Return,                          // 00EE

    // only touches PC, used for checking. skips +4 in PC due to being on top of fetch +2 move
    Skip { condition: SkipCondition }, // 3XNN 4XNN 5XY0 9XY0 EX9E EXA1

    // write to register
    SetRegister { x: usize, value: u8 }, // 6XNN
    Random { x: usize, mask: u8 },       // CXNN
    GetDelayTimer { x: usize },          // FX07

    // alu, lhs + rhs write to dst
    AddByte { x: usize, value: u8 },       // 7XNN
    Alu { op: AluOp, x: usize, y: usize }, // 8XY_

    // writes to index register (i_reg)
    SetIndex { address: u16 },   // ANNN
    AddToIndex { x: usize },     // FX1E
    SetIndexToFont { x: usize }, // FX29

    // all point to RAM starting at i_reg
    StoreBcd { x: usize },       // FX33
    StoreRegisters { x: usize }, // FX55
    LoadRegisters { x: usize },  // FX65

    // timer
    SetDelayTimer { x: usize }, // FX15
    SetSoundTimer { x: usize }, // FX18

    // wait for user io
    WaitForKey { x: usize }, // FX0A
}

#[derive(Debug, Clone, Copy)]
pub enum SkipCondition {
    EqualByte { x: usize, value: u8 },       // 3XNN
    NotEqualByte { x: usize, value: u8 },    // 4XNN
    EqualRegister { x: usize, y: usize },    // 5XY0
    NotEqualRegister { x: usize, y: usize }, // 9XY0
    KeyPressed { x: usize },                 // EX9E
    KeyNotPressed { x: usize },              // EXA1
}

#[derive(Debug, Clone, Copy)]
pub enum AluOp {
    Copy,            // 8XY0
    Or,              // 8XY1
    And,             // 8XY2
    Xor,             // 8XY3
    AddWithCarry,    // 8XY4
    Subtract,        // 8XY5
    ShiftRight,      // 8XY6
    SubtractReverse, // 8XY7
    ShiftLeft,       // 8XYE
}

impl Instruction {
    pub fn decode(opcode: u16, pc: usize) -> Result<Self, DecodeError> {
        let x = ((opcode >> 8) & 0x000F) as usize; // move 2nd nibble to end, do bitwise & to extract
        let y = ((opcode >> 4) & 0x000F) as usize; // move 3rd nibble to end, do bitwise & to extract
        let low_byte = (opcode & 0x00FF) as u8; // bitwise & to extract byte from last two nibbles
        let address = opcode & 0x0FFF; // bitwise & to extract last 3 nibbles as address dst

        // match by full opcode here for 00E0 because need full code, others can be distinguished
        // only by leading nibble
        match opcode {
            0x00E0 => Ok(Instruction::ClearScreen),

            _ => match opcode & 0xF000 {
                0x1000 => Ok(Instruction::Jump { address }),
                0x6000 => Ok(Instruction::SetRegister { x, value: low_byte }),
                0x7000 => Ok(Instruction::AddByte { x, value: low_byte }),
                0xA000 => Ok(Instruction::SetIndex { address }),
                0xD000 => Ok(Instruction::Draw {
                    x,
                    y,
                    height: (opcode & 0x000F) as usize,
                }),

                _ => Err(DecodeError::UnknownOpcode { opcode, pc }),
            },
        }
    }
}

#[derive(Debug)]
pub enum DecodeError {
    UnknownOpcode { opcode: u16, pc: usize }, // take in opcode AND pc to know exactly where error occured
}

// had to just use AI for this bc wtf is rust error handling
impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DecodeError::UnknownOpcode { opcode, pc } => {
                write!(f, "Unknown opcode {opcode:#06X} at address {pc:#05X}")
            }
        }
    }
}

impl std::error::Error for DecodeError {}
