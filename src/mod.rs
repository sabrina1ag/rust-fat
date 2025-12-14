pub mod boot;
pub mod fat;
pub mod fat_table;
pub mod cluster;
pub mod directory;
pub mod entry;
pub mod path;

pub use boot::BootSector;
pub use fat_table::FatTable;
pub use fat::Fat32Fs;
pub use cluster::ClusterChain;
pub use directory::Directory;
pub use entry::{DirEntry, DirectoryEntry, LongFileNameEntry};
pub use path::{Path, PathBuf, PathError};

use alloc::vec::Vec;
use alloc::string::String;

/// Main filesystem trait
pub trait FileSystem {
    /// List files and directories in the given path
    fn list(&self, path: &str) -> Result<Vec<DirEntry>, FileSystemError>;
    
    /// Read file contents from the given path
    fn read_file(&self, path: &str) -> Result<Vec<u8>, FileSystemError>;
    
    /// Change current directory
    fn cd(&mut self, path: &str) -> Result<(), FileSystemError>;
    
    /// Get current directory path
    fn pwd(&self) -> String;
    
    /// Create a new file at the given path
    fn create_file(&mut self, path: &str) -> Result<(), FileSystemError>;
    
    /// Write data to a file at the given path
    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), FileSystemError>;
}

/// Filesystem errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSystemError {
    /// Invalid path
    InvalidPath(String),
    /// File not found
    FileNotFound(String),
    /// Directory not found
    DirectoryNotFound(String),
    /// Invalid FAT structure
    InvalidFat(String),
    /// Invalid boot sector
    InvalidBootSector(String),
    /// Cluster chain error
    ClusterChainError(String),
    /// Directory entry error
    DirectoryEntryError(String),
    /// I/O error
    IoError(String),
    /// Out of memory
    OutOfMemory,
    /// Unsupported feature
    Unsupported(String),
}

impl core::fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FileSystemError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
            FileSystemError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            FileSystemError::DirectoryNotFound(msg) => write!(f, "Directory not found: {}", msg),
            FileSystemError::InvalidFat(msg) => write!(f, "Invalid FAT: {}", msg),
            FileSystemError::InvalidBootSector(msg) => write!(f, "Invalid boot sector: {}", msg),
            FileSystemError::ClusterChainError(msg) => write!(f, "Cluster chain error: {}", msg),
            FileSystemError::DirectoryEntryError(msg) => write!(f, "Directory entry error: {}", msg),
            FileSystemError::IoError(msg) => write!(f, "I/O error: {}", msg),
            FileSystemError::OutOfMemory => write!(f, "Out of memory"),
            FileSystemError::Unsupported(msg) => write!(f, "Unsupported: {}", msg),
        }
    }
}

impl From<PathError> for FileSystemError {
    fn from(err: PathError) -> Self {
        let msg = match err {
            PathError::InvalidFormat(s) => s,
            PathError::Empty => "Empty path".into(),
            PathError::ComponentTooLong => "Path component too long".into(),
        };
        FileSystemError::InvalidPath(msg)
    }
}
