// CLI requires std for I/O operations
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
fn main() {
    use std::io::{self, Write};
    use std::fs;
    use mini_fat32::{Fat32Fs, FileSystem, FileSystemError};
    
    println!("Mini-FAT32 CLI");
    println!("==============");
    
    // Load filesystem image
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <fat32_image>", args[0]);
        std::process::exit(1);
    }
    
    let image_path = &args[1];
    let device_data = match fs::read(image_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading image: {}", e);
            std::process::exit(1);
        }
    };
    
    unsafe {
        let mut fs = match Fat32Fs::new(&device_data) {
            Ok(fs) => fs,
            Err(e) => {
                eprintln!("Error initializing filesystem: {}", e);
                std::process::exit(1);
            }
        };
        
        println!("Filesystem loaded successfully!");
        println!("Current directory: {}", fs.pwd());
        println!("\nCommands: ls <path>, cat <path>, cd <path>, pwd, exit");
        println!("Type 'help' for more information\n");
        
        loop {
            print!("fat32> ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();
                    if input.is_empty() {
                        continue;
                    }
                    
                    let parts: Vec<&str> = input.split_whitespace().collect();
                    if parts.is_empty() {
                        continue;
                    }
                    
                    match parts[0] {
                        "ls" => {
                            let path = if parts.len() > 1 { parts[1] } else { "." };
                            match fs.list(path) {
                                Ok(entries) => {
                                    if entries.is_empty() {
                                        println!("(empty)");
                                    } else {
                                        for entry in entries {
                                            match entry.name() {
                                                Ok(name) => {
                                                    let marker = if entry.is_directory() { "/" } else { "" };
                                                    println!("{}{}", name, marker);
                                                }
                                                Err(e) => println!("<error: {}>", e),
                                            }
                                        }
                                    }
                                }
                                Err(e) => eprintln!("Error: {}", e),
                            }
                        }
                        "cat" => {
                            if parts.len() < 2 {
                                eprintln!("Usage: cat <file>");
                                continue;
                            }
                            match fs.read_file(parts[1]) {
                                Ok(data) => {
                                    let data_len = data.len();
                                    match String::from_utf8(data) {
                                        Ok(text) => print!("{}", text),
                                        Err(_) => println!("<binary data, {} bytes>", data_len),
                                    }
                                }
                                Err(e) => eprintln!("Error: {}", e),
                            }
                        }
                        "cd" => {
                            let path = if parts.len() > 1 { parts[1] } else { "/" };
                            match fs.cd(path) {
                                Ok(_) => {
                                    // Success
                                }
                                Err(e) => eprintln!("Error: {}", e),
                            }
                        }
                        "pwd" => {
                            println!("{}", fs.pwd());
                        }
                        "exit" | "quit" | "q" => {
                            println!("Goodbye!");
                            break;
                        }
                        "help" => {
                            println!("Available commands:");
                            println!("  ls [path]     - List directory contents");
                            println!("  cat <file>    - Read and display file");
                            println!("  cd [path]     - Change directory");
                            println!("  pwd           - Print current directory");
                            println!("  exit/quit/q   - Exit CLI");
                            println!("  help          - Show this help");
                        }
                        _ => {
                            eprintln!("Unknown command: {}. Type 'help' for help.", parts[0]);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    break;
                }
            }
        }
    }
}

#[cfg(not(feature = "std"))]
fn main() {
    // For no_std, CLI is not available
    // Use the library API directly instead
    panic!("CLI requires std feature. Use the library API in no_std mode.");
}
