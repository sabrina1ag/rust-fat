use crate::fs::FileSystemError;
use alloc::string::String;
use alloc::vec::Vec;

/// Directory entry (short name format - 32 bytes)
#[repr(C, packed)]
pub struct DirectoryEntry {
    /// Short name (8.3 format)
    pub name: [u8; 11],
    /// Attributes
    pub attributes: u8,
    /// Reserved (NT)
    pub nt_reserved: u8,
    /// Creation time (tenths of second)
    pub creation_time_tenths: u8,
    /// Creation time
    pub creation_time: u16,
    /// Creation date
    pub creation_date: u16,
    /// Last access date
    pub last_access_date: u16,
    /// First cluster high (bits 16-31)
    pub first_cluster_high: u16,
    /// Last write time
    pub last_write_time: u16,
    /// Last write date
    pub last_write_date: u16,
    /// First cluster low (bits 0-15)
    pub first_cluster_low: u16,
    /// File size in bytes
    pub file_size: u32,
}

impl DirectoryEntry {
    /// Parse directory entry from raw bytes
    /// 
    /// # Safety
    /// 
    /// This function uses unsafe to transmute bytes into the DirectoryEntry struct.
    /// It is safe if the input slice is exactly 32 bytes. The packed representation
    /// ensures correct alignment. The caller must ensure the data is valid.
    pub unsafe fn from_bytes(data: &[u8]) -> Result<Self, FileSystemError> {
        if data.len() < 32 {
            return Err(FileSystemError::DirectoryEntryError(
                "Directory entry must be at least 32 bytes".into()
            ));
        }
        
        // Safety: We verify the size and use a packed struct
        let entry = core::ptr::read(data.as_ptr() as *const DirectoryEntry);
        
        // Check if entry is free (0x00) or deleted (0xE5)
        if entry.name[0] == 0x00 || entry.name[0] == 0xE5 {
            return Err(FileSystemError::DirectoryEntryError(
                "Empty or deleted entry".into()
            ));
        }
        
        Ok(entry)
    }
    
    /// Check if entry is a directory
    pub fn is_directory(&self) -> bool {
        (self.attributes & 0x10) != 0
    }
    
    /// Check if entry is a file
    pub fn is_file(&self) -> bool {
        !self.is_directory() && (self.attributes & 0x08) == 0
    }
    
    /// Check if entry is a volume label
    pub fn is_volume_label(&self) -> bool {
        (self.attributes & 0x08) != 0
    }
    
    /// Get first cluster number
    pub fn first_cluster(&self) -> u32 {
        ((self.first_cluster_high as u32) << 16) | (self.first_cluster_low as u32)
    }
    
    /// Get file size
    pub fn file_size(&self) -> u32 {
        self.file_size
    }
    
    /// Get short name as string (8.3 format)
    pub fn short_name(&self) -> Result<String, FileSystemError> {
        let mut name_bytes = Vec::new();
        
        // Extract name (8 characters)
        let name_part = &self.name[0..8];
        for &b in name_part.iter() {
            if b == 0x20 {
                break;
            }
            if b != 0x20 {
                name_bytes.push(b);
            }
        }
        
        // Extract extension (3 characters)
        let ext_part = &self.name[8..11];
        let mut ext_bytes = Vec::new();
        for &b in ext_part.iter() {
            if b != 0x20 {
                ext_bytes.push(b);
            }
        }
        
        let name_str = String::from(
            core::str::from_utf8(&name_bytes)
                .map_err(|_| FileSystemError::DirectoryEntryError("Invalid UTF-8 in name".into()))?
        );
        
        if !ext_bytes.is_empty() {
            let ext_str = String::from(
                core::str::from_utf8(&ext_bytes)
                    .map_err(|_| FileSystemError::DirectoryEntryError("Invalid UTF-8 in extension".into()))?
            );
            // Manual string concatenation for no_std
            let mut result = String::new();
            result.push_str(&name_str);
            result.push('.');
            result.push_str(&ext_str);
            Ok(result)
        } else {
            Ok(name_str)
        }
    }
}

/// Long File Name (LFN) entry
#[repr(C, packed)]
pub struct LongFileNameEntry {
    /// Sequence number and flags
    pub sequence: u8,
    /// Name characters 1-5 (10 bytes, UTF-16 LE)
    pub name1: [u16; 5],
    /// Attributes (must be 0x0F)
    pub attributes: u8,
    /// Type (must be 0x00)
    pub type_: u8,
    /// Checksum of short name
    pub checksum: u8,
    /// Name characters 6-11 (12 bytes, UTF-16 LE)
    pub name2: [u16; 6],
    /// First cluster (must be 0x0000)
    pub first_cluster: u16,
    /// Name characters 12-13 (4 bytes, UTF-16 LE)
    pub name3: [u16; 2],
}

impl LongFileNameEntry {
    /// Check if this is a valid LFN entry
    pub fn is_valid(&self) -> bool {
        self.attributes == 0x0F && self.type_ == 0x00 && self.first_cluster == 0x0000
    }
    
    /// Get sequence number
    pub fn sequence_number(&self) -> u8 {
        self.sequence & 0x3F
    }
    
    /// Check if this is the last LFN entry
    pub fn is_last(&self) -> bool {
        (self.sequence & 0x40) != 0
    }
    
    /// Extract name characters from this entry
    pub fn name_chars(&self) -> Vec<u16> {
        let mut chars = Vec::new();
        // Copy to avoid unaligned reference issues with packed struct
        let name1 = self.name1;
        let name2 = self.name2;
        let name3 = self.name3;
        chars.extend_from_slice(&name1);
        chars.extend_from_slice(&name2);
        chars.extend_from_slice(&name3);
        chars
    }
}

/// Directory entry with optional long name
pub struct DirEntry {
    /// Short name entry
    pub entry: DirectoryEntry,
    /// Optional long file name
    pub long_name: Option<String>,
}

impl DirEntry {
    /// Create a new DirEntry from a DirectoryEntry
    pub fn new(entry: DirectoryEntry) -> Self {
        Self {
            entry,
            long_name: None,
        }
    }
    
    /// Set long file name
    pub fn with_long_name(mut self, long_name: String) -> Self {
        self.long_name = Some(long_name);
        self
    }
    
    /// Get display name (long name if available, otherwise short name)
    pub fn name(&self) -> Result<String, FileSystemError> {
        if let Some(ref long_name) = self.long_name {
            Ok(long_name.clone())
        } else {
            self.entry.short_name()
        }
    }
    
    /// Check if entry is a directory
    pub fn is_directory(&self) -> bool {
        self.entry.is_directory()
    }
    
    /// Check if entry is a file
    pub fn is_file(&self) -> bool {
        self.entry.is_file()
    }
    
    /// Get first cluster
    pub fn first_cluster(&self) -> u32 {
        self.entry.first_cluster()
    }
    
    /// Get file size
    pub fn file_size(&self) -> u32 {
        self.entry.file_size()
    }
}
