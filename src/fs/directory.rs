use crate::fs::FileSystemError;
use crate::fs::entry::{DirectoryEntry, DirEntry, LongFileNameEntry};
use crate::fs::cluster::ClusterChain;
use alloc::vec::Vec;
use alloc::string::String;

/// Directory management
pub struct Directory;

impl Directory {
    /// Read all entries from a directory cluster chain
    /// 
    /// # Safety
    /// 
    /// The cluster_chain must be valid and the data must contain valid directory entries.
    pub unsafe fn read_entries(
        _cluster_chain: &ClusterChain,
        data: &[u8],
    ) -> Result<Vec<DirEntry>, FileSystemError> {
        let mut entries = Vec::new();
        let mut lfn_parts: Vec<(u8, Vec<u16>)> = Vec::new();
        
        // Parse entries (32 bytes each)
        for chunk in data.chunks_exact(32) {
            if chunk[0] == 0x00 {
                // End of directory
                break;
            }
            
            if chunk[0] == 0xE5 {
                // Deleted entry, skip
                lfn_parts.clear();
                continue;
            }
            
            // Check if this is a Long File Name entry
            if chunk[11] == 0x0F {
                // Safety: This is a valid LFN entry structure
                let lfn = core::ptr::read(chunk.as_ptr() as *const LongFileNameEntry);
                if lfn.is_valid() {
                    let seq = lfn.sequence_number();
                    let chars = lfn.name_chars();
                    lfn_parts.push((seq, chars));
                    
                    if lfn.is_last() {
                        // Sort by sequence number (descending)
                        lfn_parts.sort_by(|a, b| b.0.cmp(&a.0));
                        // Reconstruct long name
                        let mut long_name = String::new();
                        for (_, chars) in &lfn_parts {
                            for &ch in chars {
                                if ch == 0 || ch == 0xFFFF {
                                    break;
                                }
                                // Convert UTF-16 to char (simplified)
                                if ch < 0x80 {
                                    long_name.push(ch as u8 as char);
                                }
                            }
                        }
                        
                        // Next entry should be the short name entry
                        continue;
                    }
                }
                continue;
            }
            
            // Regular directory entry
            match DirectoryEntry::from_bytes(chunk) {
                Ok(entry) => {
                    // Skip volume labels
                    if entry.is_volume_label() {
                        lfn_parts.clear();
                        continue;
                    }
                    
                    let mut dir_entry = DirEntry::new(entry);
                    
                    // If we have LFN parts, use them
                    if !lfn_parts.is_empty() {
                        let mut long_name = String::new();
                        for (_, chars) in &lfn_parts {
                            for &ch in chars {
                                if ch == 0 || ch == 0xFFFF {
                                    break;
                                }
                                if ch < 0x80 {
                                    long_name.push(ch as u8 as char);
                                }
                            }
                        }
                        if !long_name.is_empty() {
                            dir_entry = dir_entry.with_long_name(long_name);
                        }
                        lfn_parts.clear();
                    }
                    
                    entries.push(dir_entry);
                }
                Err(_) => {
                    lfn_parts.clear();
                    continue;
                }
            }
        }
        
        Ok(entries)
    }
    
    /// Find an entry by name in directory data
    pub fn find_entry(
        data: &[u8],
        name: &str,
    ) -> Result<Option<DirectoryEntry>, FileSystemError> {
        // Parse entries (32 bytes each)
        for chunk in data.chunks_exact(32) {
            if chunk[0] == 0x00 {
                break;
            }
            if chunk[0] == 0xE5 {
                continue;
            }
            
            unsafe {
                match DirectoryEntry::from_bytes(chunk) {
                    Ok(entry) => {
                        match entry.short_name() {
                            Ok(entry_name) => {
                                if entry_name == name || entry_name.eq_ignore_ascii_case(name) {
                                    return Ok(Some(entry));
                                }
                            }
                            Err(_) => continue,
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
        
        Ok(None)
    }
}
