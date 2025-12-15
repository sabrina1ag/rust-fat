#![no_std]
#![no_main]

extern crate alloc;

use fat32_fs::cli::{Cli, Command};
use fat32_fs::fs::FileSystemError;

// Note: In a real no_std environment, you would need to provide
// custom panic handler and allocator. This is a placeholder.

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Placeholder entry point
    // In a real implementation, this would:
    // 1. Initialize allocator
    // 2. Load filesystem data
    // 3. Create CLI instance
    // 4. Run command loop
    
    loop {
        // Infinite loop - in real implementation, handle commands
        core::hint::spin_loop();
    }
}

// Example usage (commented out - requires std for now):
/*
fn main() {
    // Load filesystem data from somewhere
    let device_data = include_bytes!("../../test_fat32.img");
    
    unsafe {
        match Cli::new(device_data) {
            Ok(mut cli) => {
                // Example commands
                let commands = vec![
                    "pwd",
                    "ls",
                    "cd /",
                    "ls",
                    "cat /readme.txt",
                ];
                
                for cmd_str in commands {
                    let cmd = Command::parse(cmd_str);
                    match cli.execute(cmd) {
                        Ok(output) => println!("{}", output),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("Failed to initialize filesystem: {}", e),
        }
    }
}
*/

