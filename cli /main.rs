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
