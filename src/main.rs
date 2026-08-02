// CHIP-8 emulator — entry point.
//
// This is intentionally empty. The architecture (modules, the machine struct,
// the fetch-decode-execute loop, the opcode representation) is YOURS to design
// and build ticket by ticket. See README.md for the project brief and the
// milestone backlog you'll turn into user stories.
//
// Nothing here is wired up on purpose — the learning is in the wiring.

use std::fs;

fn main() {
    let mut machine = Chip8::new(); // syntax is :: and not '.new()' because we are calling on type of
                                // Chip8 and not on value of anything
    let rom = fs::read("roms/ibm-logo.ch8").unwrap();  // owned vs borrow here, we use &rom in line 14 to signify this
    machine.load_rom(&rom); // we are 'borrowing' the data, i.e. not responsibile for its lifetime
                            // without this, if we passed rom into a function once it was over it
                            // would become unusable because the memory would be freed
    println!("First two bytes {} {}, Last two bytes {} {}", machine.memory[0x200], machine.memory[0x201], machine.memory[0x282], machine.memory[0x283]); 
}   // make sure numbers are in same base, 132 byte file (decimal), but indexes are in hex so
    // 200+132 does not give last address of memory, must either 
    // A. move 0x200 to decimal (512) and do 512 + 132 - 1 = 643, then convert back to hex (0x283)
    // B. move 132 to hex (0x84) and add 0x200 + 0x84 - 1 and get 0x283

// rust compiler knows [u8; 2], rom array length is 2 at compile time.
// this then gets coerced into a pointer to first byte and length slice reference that is converted
// to a &[u8] chunk of memory (no length passed around)

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
            self.memory[0x200 + i] = *byte; // start at 0x200 manually here, since we are not in FDE
                                           // .loop, use PC when entering fetching stage and
                                           // increment by 2 each time because of grabbing memory
                                           // in 2-byte chunks
        }
    }
}
