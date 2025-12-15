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

// alloc en no_std
use alloc::vec::Vec;
use alloc::string::String;

/// Le FileSystem doit savoir faire ça
pub trait FileSystem {
    // utilisé par ls, retourne contenu dossier ou code erreur
    fn list(&self, path: &str) -> Result<Vec<DirEntry>, FileSystemError>;
    
    /// lire contenu fichier ou erreur si n'existe pas ou c'est pas un fichier
    fn read_file(&self, path: &str) -> Result<Vec<u8>, FileSystemError>;
    
    /// Changer Repertoire courant, modifier la variable current_patj
    fn cd(&mut self, path: &str) -> Result<(), FileSystemError>;
    
    /// le chemin courant sous forme de string, ne peut pas echouer c'est un affichage
    fn pwd(&self) -> String;
    
    /// pas utilisé vu que creation fichier ne marche pas :)
    fn create_file(&mut self, path: &str) -> Result<(), FileSystemError>;
    
    /// pas utilisé vu que ecrire dans un fichier ne marche pas :)
    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), FileSystemError>;
}

/// Toutes les erreurs possibles du FS
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSystemError {
    /// Erreurs Logiques 

    InvalidPath(String),

    FileNotFound(String),
    
    DirectoryNotFound(String),
 
    InvalidFat(String),
   
    InvalidBootSector(String),

    ClusterChainError(String),
  
    DirectoryEntryError(String),
    /// Erreur pures IO et Unsupported
    IoError(String),
    
    OutOfMemory,
    
    Unsupported(String),
}
// utile pour CLI pour faire println(, error)
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
// meme si path::new retourne patherror, on le convertit en filesystem error
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
