use std::io::{self, Write};

pub fn render(display: &[[u8; 32]; 64]) {
    print!("\x1b[H");
    for y in 0..32 {
        for x in 0..64 {
            if display[x][y] == 1 {
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
