use crate::{display, instruction::DecodeError};
use std::cmp::min;

use crate::instruction::Instruction;

// font set for rendering
const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const FONT_HEAD_ADDRESS: usize = 0x50;

// indexes used to access array set to usize to avoid constant casting to usize

pub struct Chip8 {
    memory: [u8; 4096],
    register: [u8; 16],
    i_reg: u16,
    program_counter: usize,
    stack: [u16; 16],
    stack_pointer: usize,
    delay_timer: u8,
    sound_timer: u8,
    display: [[u8; 32]; 64],
    keypad: [bool; 16],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = [0; 4096];
        for font in 0..80 {
            memory[FONT_HEAD_ADDRESS + font] = FONT_SET[font];
        }

        Self {
            memory,
            register: [0; 16],
            i_reg: 0,
            program_counter: 0x200, // memory[512]
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [[0; 32]; 64], // [x][y], 64x32 grid
            keypad: [false; 16],
        }
    }

    // since we are taking in reference to values with pointer here in 'rom', must dereference to
    // get actual value and not the mem-address

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), ExecutionError> {
        for (i, byte) in rom.iter().enumerate() {
            self.write_to_memory(0x200 + i, *byte)?;
            // start at 0x200 manually here, since we are not in FDE
            // .loop, use PC when entering fetching stage and
            // increment by 2 each time because of grabbing memory
            // in 2-byte chunks
        }
        Ok(())
    }

    fn fetch(&self) -> Result<u16, ExecutionError> {
        // check for program_counter <= 4094 should be here

        // combine two u8 int into a u16 by shifting 8 bits to left and concatenating the 'low'
        // bits on the end
        // no use of '+' here because it is not arithmetic (although would lead to same value), use
        // of '|' here is to signify intent of putting two bytes and two bytes together to make u16
        // opcode

        // we also must cast these to u16 to do the comparison because of the binary of u8
        // 0000 0000 and 0000 0000 both as u8 cannot be pushed together to make a u16 byte without
        // first casting each to u16 and then doing the combination
        // 0000 0000 0000 0000

        let high = self.read_memory(self.program_counter)? as u16;
        let low = self.read_memory(self.program_counter + 1)? as u16;

        Ok(high << 8 | low)
    }

    pub fn execute(&mut self, instruction: Instruction) -> Result<(), ExecutionError> {
        match instruction {
            Instruction::ClearScreen => {
                // clear screen
                self.display = [[0; 32]; 64];
            }
            Instruction::Jump { address } => {
                // jump to address NNN and execute from there
                self.program_counter = address as usize;
            }
            Instruction::SetRegister { x, value } => {
                // set register at index X to value NN
                self.register[x] = value;
            }
            Instruction::AddByte { x, value } => {
                // add value of NN to register[X]
                self.register[x] = self.register[x].wrapping_add(value);
                // prevent overflow and wrap back to 0 when value becomes >255
            }
            Instruction::SetIndex { address } => {
                // set index, used in drawing to say 'start drawing from this index'
                self.i_reg = address;
            }
            Instruction::Draw { x, y, height } => {
                // draw font sprite onto screen, read N bytes starting at memory[I], get xy pos from memory
                self.register[0xF] = 0;

                // mod used to check if value from reg is between 0 - 64, if not we take remainder
                // and use that as starting x and y position
                let xpos = (self.register[x] % 64) as usize;
                let ypos = (self.register[y] % 32) as usize;

                let row_count = min(height, 32 - ypos);

                for row in 0..row_count {
                    let sprite_byte = self.read_memory(self.i_reg as usize + row)?;

                    // xpos can be a constant here with xpos < 56 because the bit_count for a
                    // sprite will always bit 2bytes (8)
                    let mut bit_count = 8;
                    if xpos > 56 {
                        bit_count = 64 - xpos;
                    }

                    for bit in 0..bit_count {
                        if sprite_byte >> (7 - bit) & 0x01 == 1 {
                            if self.display[xpos + bit][ypos + row] == 1 {
                                self.register[0xF] = 1;
                            }

                            self.display[xpos + bit][ypos + row] ^= 1;
                        }
                    }
                }
                // get row count from last nibble of opcode, go up to count - 1
                // for each row, get sprite bytes from memory[I] (+ row offset)

                // parse these bytes by taking the total length (8, but since start at 0 subtract 1)
                // and do 7 - bit for the shift (ex. bit = 2, 7 - 2 = 5, shift 5 right to move 3rd
                // most bit to the end, do 0x01 AND bitwise trick to only get 1s)
                // check if 1, if true need to display else leave 0 and do nothing

                // for display, get xpos and ypos from opcode index (DXYN) at reg[x], reg[y]
                // using this as start cord, must get offset same as i_reg with + bit for the
                // column and + row for the row
                // then simply do XOR on display, but need to make note of collision (if that
                // display already set to 1)
            }

            _ => todo!("instruction not implemented yet: {instruction:?}"),
        }

        Ok(())
    }

    pub fn cycle(&mut self) -> Result<(), ExecutionError> {
        // capture PC before advancing so a decode error reports the address of the
        // instruction that actually failed, not the one after it
        let pc = self.program_counter;

        let opcode = self.fetch()?;
        self.program_counter += 2;

        // add on 2 here and not later because we save the opcode variable then do decode +
        // execute, which could lead to changes in PC that if we did at the end would lead to off
        // by x errors, fetch-advance-decode-execute

        // decode logic, then execute logic
        let instruction = Instruction::decode(opcode, pc)?;

        self.execute(instruction)?;

        Ok(())
    }

    pub fn render(&self) {
        display::render(&self.display);
    }

    fn write_to_memory(&mut self, index: usize, value: u8) -> Result<(), ExecutionError> {
        if index >= self.memory.len() {
            return Err(ExecutionError::MemoryOutOfBounds { index });
        }
        self.memory[index] = value;
        Ok(())
    }

    fn read_memory(&self, index: usize) -> Result<u8, ExecutionError> {
        if index >= self.memory.len() {
            return Err(ExecutionError::MemoryOutOfBounds { index });
        }
        Ok(self.memory[index])
    }
}

// creates types of errors that ExecutionError can hold (kind of like constructor)
#[derive(Debug)]
pub enum ExecutionError {
    MemoryOutOfBounds { index: usize },
    StackOverflow { stack_pointer: usize },
    StackUnderflow { stack_pointer: usize },
    Decode(DecodeError),
}

// turns type of error above into easier to understand messages
impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExecutionError::MemoryOutOfBounds { index } => {
                write!(f, "Memory out of bounds for index {index}")
            }
            ExecutionError::StackOverflow { stack_pointer } => {
                write!(
                    f,
                    "Stack Overflow, {stack_pointer} out of bounds for stack of len 16"
                )
            }
            ExecutionError::StackUnderflow { stack_pointer } => {
                write!(f, "Stack is empty at index {stack_pointer}")
            }
            ExecutionError::Decode(e) => write!(f, "{e}"),
        }
    }
}

impl From<DecodeError> for ExecutionError {
    fn from(e: DecodeError) -> Self {
        ExecutionError::Decode(e)
    }
}
impl std::error::Error for ExecutionError {}
