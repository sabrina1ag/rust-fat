use crate::fs::{Fat32Fs, FileSystem, FileSystemError};
use crate::fs::path::PathBuf;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// CLI command enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// List directory contents
    List(Option<PathBuf>),
    /// Read file contents
    Read(PathBuf),
    /// Change directory
    ChangeDirectory(PathBuf),
    /// Create file
    CreateFile(PathBuf),
    /// Write to file
    Write(PathBuf, Vec<u8>),
    /// Print current directory
    PrintWorkingDirectory,
    /// Exit
    Exit,
    /// Help
    Help,
    /// Unknown command
    Unknown(String),
}

impl Command {
    /// Parse command from input string
    pub fn parse(input: &str) -> Self {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Command::Unknown(String::new());
        }
        
        match parts[0] {
            "ls" | "list" => {
                if parts.len() > 1 {
                    match PathBuf::new(parts[1]) {
                        Ok(path) => Command::List(Some(path)),
                        Err(_) => Command::Unknown(format!("Invalid path: {}", parts[1])),
                    }
                } else {
                    Command::List(None)
                }
            }
            "cat" | "read" => {
                if parts.len() > 1 {
                    match PathBuf::new(parts[1]) {
                        Ok(path) => Command::Read(path),
                        Err(_) => Command::Unknown(format!("Invalid path: {}", parts[1])),
                    }
                } else {
                    Command::Unknown("Missing file path".into())
                }
            }
            "cd" => {
                if parts.len() > 1 {
                    match PathBuf::new(parts[1]) {
                        Ok(path) => Command::ChangeDirectory(path),
                        Err(_) => Command::Unknown(format!("Invalid path: {}", parts[1])),
                    }
                } else {
                    Command::ChangeDirectory(PathBuf::root())
                }
            }
            "pwd" => Command::PrintWorkingDirectory,
            "create" => {
                if parts.len() > 1 {
                    match PathBuf::new(parts[1]) {
                        Ok(path) => Command::CreateFile(path),
                        Err(_) => Command::Unknown(format!("Invalid path: {}", parts[1])),
                    }
                } else {
                    Command::Unknown("Missing file path".into())
                }
            }
            "write" => {
                if parts.len() > 2 {
                    match PathBuf::new(parts[1]) {
                        Ok(path) => {
                            let data = parts[2..].join(" ").into_bytes();
                            Command::Write(path, data)
                        }
                        Err(_) => Command::Unknown(format!("Invalid path: {}", parts[1])),
                    }
                } else {
                    Command::Unknown("Missing file path or data".into())
                }
            }
            "exit" | "quit" | "q" => Command::Exit,
            "help" | "?" => Command::Help,
            _ => Command::Unknown(format!("Unknown command: {}", parts[0])),
        }
    }
}

/// CLI handler
pub struct Cli {
    /// Filesystem instance
    fs: Fat32Fs,
}

impl Cli {
    /// Create a new CLI instance
    /// 
    /// # Safety
    /// 
    /// The device_data must be valid FAT32 filesystem data.
    pub unsafe fn new(device_data: &[u8]) -> Result<Self, FileSystemError> {
        let fs = Fat32Fs::new(device_data)?;
        Ok(Self { fs })
    }
    
    /// Execute a command
    pub fn execute(&mut self, command: Command) -> Result<String, FileSystemError> {
        match command {
            Command::List(path) => {
                let target_path = path
                    .map(|p| p.as_path().clone())
                    .unwrap_or_else(|| self.fs.current_directory().clone());
                
                let entries = self.fs.list(&target_path)?;
                let mut output = String::new();
                for entry in entries {
                    // TODO: Format directory entries properly
                    output.push_str(&format!("{:?}\n", entry));
                }
                Ok(output)
            }
            Command::Read(path) => {
                let data = self.fs.read_file(path.as_path())?;
                let data_len = data.len();
                match String::from_utf8(data) {
                    Ok(s) => Ok(s),
                    Err(_) => Ok(format!("<binary data, {} bytes>", data_len)),
                }
            }
            Command::ChangeDirectory(path) => {
                self.fs.cd(path.as_path())?;
                Ok(format!("Changed to: {}", self.fs.current_directory().to_string()))
            }
            Command::PrintWorkingDirectory => {
                Ok(self.fs.current_directory().to_string())
            }
            Command::CreateFile(path) => {
                self.fs.create_file(path.as_path())?;
                Ok(format!("Created file: {}", path.to_string()))
            }
            Command::Write(path, data) => {
                self.fs.write_file(path.as_path(), &data)?;
                Ok(format!("Wrote {} bytes to: {}", data.len(), path.to_string()))
            }
            Command::Help => {
                Ok(String::from(
                    "Available commands:\n\
                     ls [path]          - List directory contents\n\
                     cat <path>         - Read file contents\n\
                     cd [path]          - Change directory\n\
                     pwd                - Print current directory\n\
                     create <path>      - Create a new file\n\
                     write <path> <data> - Write data to file\n\
                     help               - Show this help\n\
                     exit               - Exit the CLI"
                ))
            }
            Command::Exit => {
                Ok(String::from("Exiting..."))
            }
            Command::Unknown(msg) => {
                Err(FileSystemError::InvalidPath(msg))
            }
        }
    }
    
    /// Get filesystem reference
    pub fn filesystem(&self) -> &Fat32Fs {
        &self.fs
    }
    
    /// Get mutable filesystem reference
    pub fn filesystem_mut(&mut self) -> &mut Fat32Fs {
        &mut self.fs
    }
}
