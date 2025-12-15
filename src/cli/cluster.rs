use crate::fs::FileSystemError;
use crate::fs::fat_table::FatTable; //on utilise fat table
use alloc::vec::Vec;

/// Cluster chain for traversing file/directory data
pub struct ClusterChain {
    /// List of cluster numbers in the chain
    clusters: Vec<u32>,
}

impl ClusterChain {
    /// Create a new cluster chain starting from the given cluster
    /// 
    /// # Safety
    /// 
    /// The fat_table must be valid and the start_cluster must be a valid cluster number.
    /// This function will traverse the FAT to build the complete chain.
    pub fn new(
        fat_table: &FatTable,
        start_cluster: u32,
    ) -> Result<Self, FileSystemError> {
        if start_cluster < 2 {
            return Err(FileSystemError::ClusterChainError(
                "Invalid cluster number (must be >= 2)".into()
            ));
        }
        
        let mut clusters = Vec::new();
        let mut current = start_cluster;
        
        // Maximum chain length to prevent infinite loops
        let max_clusters = fat_table.len();
        let mut iterations = 0;
        
        loop {
            if iterations >= max_clusters {
                return Err(FileSystemError::ClusterChainError(
                    "Cluster chain too long or circular".into()
                ));
            }
            
            clusters.push(current);
            
            // Get next cluster from FAT
            let next = fat_table.get_entry(current)?;
            
            // Check for end of chain markers
            if next >= 0x0FFFFFF8 {
                // End of chain
                break;
            }
            
            if next == 0x0FFFFFF7 {
                return Err(FileSystemError::ClusterChainError(
                    "Bad cluster in chain".into()
                ));
            }
            
            if next < 2 {
                return Err(FileSystemError::ClusterChainError(
                    "Invalid next cluster number".into()
                ));
            }
            
            current = next;
            iterations += 1;
        }
        
        Ok(Self { clusters })
    }
    
    /// Get all cluster numbers in the chain
    pub fn clusters(&self) -> &[u32] {
        &self.clusters
    }
    
    /// Get the number of clusters in the chain
    pub fn len(&self) -> usize {
        self.clusters.len()
    }
    
    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.clusters.is_empty()
    }
    
    /// Calculate total size in bytes
    pub fn total_size(&self, cluster_size: u32) -> u32 {
        (self.clusters.len() as u32) * cluster_size
    }
}
