// CHIP-8 emulator — entry point.
//
// This is intentionally empty. The architecture (modules, the machine struct,
// the fetch-decode-execute loop, the opcode representation) is YOURS to design
// and build ticket by ticket. See README.md for the project brief and the
// milestone backlog you'll turn into user stories.
//
// Nothing here is wired up on purpose — the learning is in the wiring.

use std::fs;
use std::thread;
use std::time::Duration;
use std::io::{self, Write};

fn main() {
    
    // syntax is :: and not '.new()' because we are calling on type of
    // Chip8 and not on value of anything
    
    let mut machine = Chip8::new(); 
    let rom = fs::read("roms/ibm-logo.ch8").unwrap();  
   
    // owned vs borrow here, we use &rom to signify this
    
    machine.load_rom(&rom);

    // we are 'borrowing' the data, i.e. not responsibile for its lifetime
    // without this, if we passed rom into a function once it was over it
    // would become unusable because the memory would be freed
    
    for _ in 0..20 {
        machine.cycle();
        machine.render();
        thread::sleep(Duration::from_millis(150));
    }
}

// make sure numbers are in same base, 132 byte file (decimal), but indexes are in hex so
// 200+132 does not give last address of memory, must either 
// A. move 0x200 to decimal (512) and do 512 + 132 - 1 = 643, then convert back to hex (0x283)
// B. move 132 to hex (0x84) and add 0x200 + 0x84 - 1 and get 0x283

// rust compiler knows [u8; 2], array length is 2 at compile time. (ONLY IF LENGTH SET)
// this then gets coerced into a pointer to first byte and length slice reference that is converted
// to a &[u8] chunk of memory (fat pointer that carries pointer, length instead of just pointer)

// indexes used to access array set to usize to avoid constant casting to usize

struct Chip8 {
    memory: [u8; 4096],
    register: [u8; 16],
    i_reg: u16,
    program_counter: usize,
    stack: [u16; 16],
    stack_pointer: usize,
    delay_timer: u8,
    sound_timer: u8,
    display: [[u8; 32]; 64]
}

impl Chip8 {
    fn new() -> Self {
       Self {
           memory: [0; 4096],
           register: [0; 16],
           i_reg: 0,
           program_counter: 0x200,  // memory[512]
           stack: [0; 16],
           stack_pointer: 0,
           delay_timer: 0,
           sound_timer: 0,
           display: [[0; 32]; 64] // [x][y], 64x32 grid
                                   
       } 
    }
    
    // since we are taking in reference to values with pointer here in 'rom', must dereference to
    // get actual value and not the mem-address

    fn load_rom(&mut self, rom: &[u8]) {
        for (i, byte) in rom.iter().enumerate() {
            self.memory[0x200 + i] = *byte; 
            // start at 0x200 manually here, since we are not in FDE
            // .loop, use PC when entering fetching stage and
            // increment by 2 each time because of grabbing memory
            // in 2-byte chunks
        }
    }

    fn cycle(&mut self) { 

        // combine two u8 int into a u16 by shifting 8 bits to left and concatenating the 'low'
        // bits on the end
        // no use of '+' here because it is not arithmetic (although would lead to same value), use
        // of '|' here is to signify intent of putting two bytes and two bytes together to make u16
        // opcode

        // we also must cast these to u16 to do the comparison because of the binary of u8
        // 0000 0000 and 0000 0000 both as u8 cannot be pushed together to make a u16 byte without
        // first casting each to u16 and then doing the combination
        // 0000 0000 0000 0000

        let high_byte = self.memory[self.program_counter] as u16;
        let low_byte = self.memory[self.program_counter + 1] as u16;

        let opcode = high_byte << 8 | low_byte;
        self.program_counter += 2;

        // decode logic, then execute logic
        // add on 2 here and not later because we save the opcode variable then do decode +
        // execute, which could lead to changes in PC that if we did at the end would lead to off
        // by x errors, fetch-advance-decode-execute
        
        // opcode is u16 (0000 x4)
        // this shift right takes each bit and shifts it right, with the bits that fall off just
        // being discarded
        // now ending up with 0000 0000 0000 (opcode)
        // these leading zeros are truncated off and we are just left with the trailing opcode
        //
        // we take these 'nibbles' and match them to their instruction
        // however, 0 is an outlier because there are multiple instructions that can start with a
        // nibble of '0', so we must take the whole opcode in this case and do work
        
        match opcode >> 12 {
            0x0 => match opcode {
                0x00E0 => self.display = [[0; 32]; 64], // clear screen
                _ => {}
            }
            0x1 => { // jump to address NNN and execute from there
                let address = (opcode & 0x0FFF) as usize;
                self.program_counter = address;
            },
            0x6 => { // set register at index X to value NN
                let index = get_reg_index_one_nibble(opcode);
                let operand = (opcode & 0xFF) as u8;
                self.register[index] = operand;
            },
            0x7 => { // add value of NN to register[X]
                let index = get_reg_index_one_nibble(opcode);
                let operand = (opcode & 0xFF) as u8;
                self.register[index] = self.register[index].wrapping_add(operand);  
                // prevent overflow and wrap back to 0 when value becomes >255
            },
            0xA => { // set index, used in drawing to say 'start drawing from this index'
                let address = opcode & 0x0FFF;
                self.i_reg = address;
                // only shift when a piece of the bits you want is not on the lower portion
            },
            0xD => { // draw font sprite onto screen, read N bytes starting at memory[I], get xy pos from memory
                self.register[0xF] = 1;
                let row_count = get_loop_count_operand(opcode);
                for row in 0..row_count { 
                    let sprite_byte = self.memory[self.i_reg as usize + row];
                    for bit in 0..8 {
                        if sprite_byte >> (7 - bit) & 0x01 == 1 {
                            let xindex = get_xindex(opcode);
                            let yindex = get_yindex(opcode);
                            
                            let xpos = self.register[xindex];
                            let ypos = self.register[yindex];
                           
                            if self.display[xpos as usize + bit][ypos as usize + row] == 1 {
                                self.register[0xF] = 1;
                            }
                            self.display[xpos as usize + bit][ypos as usize + row] ^= 1;
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
            },
            _ => println!("unknown opcode {:04X}", opcode)
        }
    }
    
    fn render(&self) {
        print!("\x1b[H");
        for y in 0..32 {
            for x in 0..64 {
                if self.display[x][y] == 1 {
                    print!("##");
                } else {
                    print!("  ");
                }
            }
            println!();
        }
        io::stdout().flush().unwrap(); 
        // force sprite to appear instantly, instead of waiting for loop
    }
}


// get the register index of an opcode that only has 1 leading nibble
// shift right 8, use bitwise & to get the address of last 4 bits for address
fn get_reg_index_one_nibble(opcode: u16) -> usize {
    let reg_index = (opcode >> 8) & 0x0F;
    return reg_index as usize;

    // how this works with bitwise &
    // shift 16 bit opcode to 8 bites, left with 0000 0000 n1 n2
    // then take bitwise & that returns 0 for any 0 value and 1 for both being 1
    // ex: 0x6108
    // shift right 8, 0x0061
    // 0000 0000 0110 0001 0x0061
    // 0000 0000 0000 1111 0x000F
    // perform bitwise & op, anything with (0,0), (1,0), or (0,1) gets assigned to 0
    // else, all other values (1,1) get 1
    // so final address to return becomes 0000 0001 (0x01) (since we casted to usize)
}    
 
// index used to read register to get x pos-cord
fn get_xindex(opcode: u16) -> usize {
    let xindex = (opcode >> 8) & 0x0F;
    return xindex as usize;
}
   
// index used to read register to get y pos-cord
fn get_yindex(opcode: u16) -> usize {
    let yindex = (opcode >> 4) & 0x00F;
    return yindex as usize;
}
    
// number of times we walk memory from draw opcode
fn get_loop_count_operand(opcode: u16) -> usize {
    let operand = opcode & 0x000F;
    return operand as usize;
}
