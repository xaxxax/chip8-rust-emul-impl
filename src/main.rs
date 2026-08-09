// CHIP-8 emulator — entry point.
//
// This is intentionally empty. The architecture (modules, the machine struct,
// the fetch-decode-execute loop, the opcode representation) is YOURS to design
// and build ticket by ticket. See README.md for the project brief and the
// milestone backlog you'll turn into user stories.
//
// Nothing here is wired up on purpose — the learning is in the wiring.

use std::error::Error;
use std::fs;
use std::thread;
use std::time::Duration;

mod cpu;
mod display;
mod instruction;

use cpu::Chip8;

fn main() -> Result<(), Box<dyn Error>> {
    // syntax is :: and not '.new()' because we are calling on type of
    // Chip8 and not on value of anything

    let mut machine = Chip8::new();
    let rom = fs::read("roms/ibm-logo.ch8")?;

    // owned vs borrow here, we use &rom to signify this

    machine.load_rom(&rom)?;

    // we are 'borrowing' the data, i.e. not responsibile for its lifetime
    // without this, if we passed rom into a function once it was over it
    // would become unusable because the memory would be freed

    for _ in 0..20 {
        if let Err(e) = machine.cycle() {
            eprintln!("{e}");
            break;
        }
        machine.render();
        thread::sleep(Duration::from_millis(150));
    }

    Ok(())
}

// make sure numbers are in same base, 132 byte file (decimal), but indexes are in hex so
// 200+132 does not give last address of memory, must either
// A. move 0x200 to decimal (512) and do 512 + 132 - 1 = 643, then convert back to hex (0x283)
// B. move 132 to hex (0x84) and add 0x200 + 0x84 - 1 and get 0x283

// rust compiler knows [u8; 2], array length is 2 at compile time. (ONLY IF LENGTH SET)
// this then gets coerced into a pointer to first byte and length slice reference that is converted
// to a &[u8] chunk of memory (fat pointer that carries pointer, length instead of just pointer)

// indexes used to access array set to usize to avoid constant casting to usize
