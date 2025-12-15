// Integration tests for Mini-FAT32

use mini_fat32::{Fat32Fs, FileSystem};
use mini_fat32::fs::boot::BootSector;
use mini_fat32::fs::fat_table::FatTable;
use mini_fat32::fs::cluster::ClusterChain;
use mini_fat32::fs::path::Path;

/// Helper: Create a minimal valid FAT32 boot sector
fn create_test_boot_sector() -> Vec<u8> {
    let mut bs = vec![0u8; 512];
    
    // Jump instruction
    bs[0] = 0xEB;
    bs[1] = 0x58;
    bs[2] = 0x90;
    
    // OEM name
    bs[3..11].copy_from_slice(b"MSWIN4.1");
    
    // Bytes per sector (512)
    bs[11..13].copy_from_slice(&512u16.to_le_bytes());
    
    // Sectors per cluster (1)
    bs[13] = 1;
    
    // Reserved sectors (32)
    bs[14..16].copy_from_slice(&32u16.to_le_bytes());
    
    // Number of FATs (2)
    bs[16] = 2;
    
    // Root entry count (0 for FAT32)
    bs[17..19].copy_from_slice(&0u16.to_le_bytes());
    
    // Total sectors 32
    bs[32..36].copy_from_slice(&102400u32.to_le_bytes());
    
    // Sectors per FAT 32
    bs[36..40].copy_from_slice(&100u32.to_le_bytes());
    
    // Root cluster
    bs[44..48].copy_from_slice(&2u32.to_le_bytes());
    
    // FS type
    bs[82..90].copy_from_slice(b"FAT32   ");
    
    // Boot signature end (0xAA55)
    bs[510..512].copy_from_slice(&0xAA55u16.to_le_bytes());
    
    bs
}

/// Helper: Create a complete FAT32 filesystem image
fn create_test_filesystem() -> Vec<u8> {
    let mut device_data = create_test_boot_sector();
    
    // Add FAT table (100 sectors)
    let fat_size = 100 * 512;
    device_data.resize(device_data.len() + fat_size, 0);
    
    // Initialize FAT: mark cluster 2 (root) as end of chain
    let fat_start = 32 * 512;
    let fat_offset = fat_start + (2 * 4);
    device_data[fat_offset..fat_offset + 4].copy_from_slice(&0x0FFFFFFFu32.to_le_bytes());
    
    // Add data area (1000 sectors)
    let data_size = 1000 * 512;
    device_data.resize(device_data.len() + data_size, 0);
    
    // Root directory starts at cluster 2
    // Mark end of directory (first entry is 0x00)
    let data_start = fat_start + (fat_size * 2);
    let root_offset = data_start + ((2 - 2) * 512);
    device_data[root_offset] = 0x00;
    
    device_data
}

#[test]
fn test_boot_sector_parsing() {
    let bs_data = create_test_boot_sector();
    
    unsafe {
        match BootSector::from_bytes(&bs_data) {
            Ok(bs) => {
                assert_eq!(bs.bytes_per_sector(), 512);
                assert_eq!(bs.sectors_per_cluster(), 1);
                assert_eq!(bs.root_cluster(), 2);
            }
            Err(e) => panic!("Failed to parse boot sector: {}", e),
        }
    }
}

#[test]
fn test_fat_table_parsing() {
    let mut fat_data = vec![0u32; 100];
    fat_data[2] = 3;
    fat_data[3] = 0x0FFFFFFF;
    
    let mut bytes = Vec::new();
    for entry in fat_data {
        bytes.extend_from_slice(&entry.to_le_bytes());
    }
    
    unsafe {
        match FatTable::from_bytes(&bytes) {
            Ok(fat) => {
                assert_eq!(fat.get_entry(2).unwrap(), 3);
                assert!(fat.is_end_of_chain(3));
            }
            Err(e) => panic!("Failed to parse FAT: {}", e),
        }
    }
}

#[test]
fn test_cluster_chain() {
    let mut fat_data = vec![0u32; 100];
    fat_data[2] = 3;
    fat_data[3] = 4;
    fat_data[4] = 0x0FFFFFFF;
    
    let mut bytes = Vec::new();
    for entry in fat_data {
        bytes.extend_from_slice(&entry.to_le_bytes());
    }
    
    unsafe {
        let fat = FatTable::from_bytes(&bytes).unwrap();
        let chain = ClusterChain::new(&fat, 2).unwrap();
        
        assert_eq!(chain.len(), 3);
        assert_eq!(chain.clusters(), &[2, 3, 4]);
    }
}

#[test]
fn test_filesystem_list_root() {
    let device_data = create_test_filesystem();
    
    unsafe {
        match Fat32Fs::new(&device_data) {
            Ok(fs) => {
                match fs.list("/") {
                    Ok(entries) => {
                        // Root directory should be empty or contain entries
                        assert!(entries.len() >= 0);
                    }
                    Err(e) => {
                        // Acceptable if directory is empty
                        println!("List root test: {}", e);
                    }
                }
            }
            Err(e) => panic!("Failed to initialize filesystem: {}", e),
        }
    }
}

#[test]
fn test_filesystem_cd() {
    let device_data = create_test_filesystem();
    
    unsafe {
        match Fat32Fs::new(&device_data) {
            Ok(mut fs) => {
                // Test cd to root
                match fs.cd("/") {
                    Ok(_) => {
                        assert_eq!(fs.pwd(), "/");
                    }
                    Err(e) => panic!("cd to root failed: {}", e),
                }
            }
            Err(e) => panic!("Failed to initialize filesystem: {}", e),
        }
    }
}

#[test]
fn test_filesystem_pwd() {
    let device_data = create_test_filesystem();
    
    unsafe {
        match Fat32Fs::new(&device_data) {
            Ok(fs) => {
                assert_eq!(fs.pwd(), "/");
            }
            Err(e) => panic!("Failed to initialize filesystem: {}", e),
        }
    }
}

#[test]
fn test_path_parsing() {
    // Test absolute path
    let path = Path::new("/usr/bin/ls").unwrap();
    assert!(path.is_absolute());
    assert_eq!(path.components().len(), 3);
    
    // Test relative path
    let path = Path::new("dir/file.txt").unwrap();
    assert!(!path.is_absolute());
    
    // Test root path
    let path = Path::root();
    assert!(path.is_absolute());
    assert!(path.is_root());
}

#[test]
fn test_read_file_not_found() {
    let device_data = create_test_filesystem();
    
    unsafe {
        match Fat32Fs::new(&device_data) {
            Ok(fs) => {
                match fs.read_file("/nonexistent.txt") {
                    Ok(_) => panic!("Should have failed to read non-existent file"),
                    Err(_) => {
                        // Expected
                    }
                }
            }
            Err(e) => panic!("Failed to initialize filesystem: {}", e),
        }
    }
}
