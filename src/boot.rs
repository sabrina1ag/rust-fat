use crate::fs::FileSystemError;

/// FAT32 Boot Sector (BPB - BIOS Parameter Block)
/// Complete structure as per FAT32 specification
#[repr(C, packed)]
pub struct BootSector {
    /// Jump instruction (3 bytes)
    pub jmp_boot: [u8; 3],
    /// OEM name (8 bytes)
    pub oem_name: [u8; 8],
    /// Bytes per sector (usually 512)
    pub bytes_per_sector: u16,
    /// Sectors per cluster
    pub sectors_per_cluster: u8,
    /// Reserved sectors
    pub reserved_sector_count: u16,
    /// Number of FATs
    pub num_fats: u8,
    /// Root entry count (unused in FAT32, should be 0)
    pub root_entry_count: u16,
    /// Total sectors (16-bit, 0 if > 65535)
    pub total_sectors_16: u16,
    /// Media descriptor
    pub media: u8,
    /// Sectors per FAT (16-bit, unused in FAT32)
    pub sectors_per_fat_16: u16,
    /// Sectors per track
    pub sectors_per_track: u16,
    /// Number of heads
    pub num_heads: u16,
    /// Hidden sectors
    pub hidden_sectors: u32,
    /// Total sectors (32-bit)
    pub total_sectors_32: u32,
    /// Sectors per FAT (32-bit)
    pub sectors_per_fat_32: u32,
    /// Flags
    pub ext_flags: u16,
    /// FAT version
    pub fat_version: u16,
    /// Root cluster number
    pub root_cluster: u32,
    /// FSInfo sector
    pub fs_info: u16,
    /// Backup boot sector
    pub backup_boot_sector: u16,
    /// Reserved (12 bytes)
    pub reserved: [u8; 12],
    /// Drive number
    pub drive_number: u8,
    /// Reserved (1 byte)
    pub reserved1: u8,
    /// Extended boot signature
    pub boot_signature: u8,
    /// Volume ID
    pub volume_id: u32,
    /// Volume label (11 bytes)
    pub volume_label: [u8; 11],
    /// File system type (8 bytes)
    pub fs_type: [u8; 8],
    /// Boot code (420 bytes)
    pub boot_code: [u8; 420],
    /// Boot sector signature (0xAA55)
    pub boot_signature_end: u16,
}

impl BootSector {
    /// Parse boot sector from raw bytes
    /// 
    /// # Safety
    /// 
    /// This function uses unsafe to transmute bytes into the BootSector struct.
    /// It is safe if the input slice is exactly 512 bytes and contains valid
    /// FAT32 boot sector data. The packed representation ensures correct alignment.
    /// The caller must ensure the data is valid FAT32 boot sector data.
    pub unsafe fn from_bytes(data: &[u8]) -> Result<Self, FileSystemError> {
        if data.len() < 512 {
            return Err(FileSystemError::InvalidBootSector(
                "Boot sector must be at least 512 bytes".into()
            ));
        }
        
        // Safety: We verify the size and use a packed struct
        // The data must be valid FAT32 boot sector data
        let bs = core::ptr::read(data.as_ptr() as *const BootSector);
        
        // Verify FAT32 signature
        if bs.fs_type[0] != b'F' || bs.fs_type[1] != b'A' || bs.fs_type[2] != b'T' || bs.fs_type[3] != b'3' {
            return Err(FileSystemError::InvalidBootSector(
                "Not a FAT32 filesystem".into()
            ));
        }
        
        if bs.boot_signature_end != 0xAA55 {
            return Err(FileSystemError::InvalidBootSector(
                "Invalid boot sector signature".into()
            ));
        }
        
        Ok(bs)
    }
    
    /// Get bytes per sector
    pub fn bytes_per_sector(&self) -> u32 {
        self.bytes_per_sector as u32
    }
    
    /// Get sectors per cluster
    pub fn sectors_per_cluster(&self) -> u32 {
        self.sectors_per_cluster as u32
    }
    
    /// Get cluster size in bytes
    pub fn cluster_size(&self) -> u32 {
        self.bytes_per_sector() * self.sectors_per_cluster()
    }
    
    /// Get FAT start sector (reserved sectors)
    pub fn fat_start_sector(&self) -> u32 {
        self.reserved_sector_count as u32
    }
    
    /// Get data area start sector
    pub fn data_start_sector(&self) -> u32 {
        self.fat_start_sector() + (self.sectors_per_fat_32 * self.num_fats as u32)
    }
    
    /// Get root cluster number
    pub fn root_cluster(&self) -> u32 {
        self.root_cluster
    }
    
    /// Get sectors per FAT (32-bit)
    pub fn sectors_per_fat(&self) -> u32 {
        self.sectors_per_fat_32
    }
    
    /// Get number of FATs
    pub fn num_fats(&self) -> u8 {
        self.num_fats
    }
}

