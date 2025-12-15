use crate::fs::FileSystemError;
use alloc::vec::Vec;d

/// FAT32 File Allocation Table
pub struct FatTable {
    /// FAT entries (each entry is 32-bit, but only 28 bits are used, 4 reserves restent les bits hauts)
    entries: Vec<u32>, // un tableau 
}

// elf.entries → un tableau avec une case par cluster du disque.
// cluster → le numéro du cluster actuel que je veux regarder.
// self.entries[cluster] → la valeur dans la FAT pour ce cluster (fin de chaine ou vide ou erreur sinon val )

impl FatTable {
    /// Parse FAT table from raw bytes
    /// 
    /// # Documentation d'une fonction Unsafe
    /// 
    /// La data doit etre valide FAT32 data de 32bit. 
    /// on utilise pas les 4 premiers bits et on doit s'assurer de l'alignement avec de call cette fonction
    pub unsafe fn from_bytes(data: &[u8]) -> Result<Self, FileSystemError> { //remplir le tableau FatTable à partir de bits bruts
        if data.len() % 4 != 0 {
            return Err(FileSystemError::InvalidFat("FAT table size must be multiple of 4".into()));
        }
        
        let mut entries = Vec::new();
        entries.reserve(data.len() / 4);
        
        for chunk in data.chunks_exact(4) {
            // FAT32 32 bits mais on utilise 24 donc masquer les 4 premiers
            let entry = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) & 0x0FFF_FFFF;
            entries.push(entry);
        }
        
        Ok(Self { entries })
    }
    
   
    /// prochain cluster ou valeur de fin de chaine
    pub fn get_entry(&self, cluster: u32) -> Result<u32, FileSystemError> {
        if cluster as usize >= self.entries.len() { //numero du cluster qu'on veut tester, self.entries vecteur de toutes les entrées FAT
            return Err(FileSystemError::InvalidFat("Cluster out of FAT bounds".into()));
        }
        
        Ok(self.entries[cluster as usize])
    }
    
    /// cluster = fin de chaine ?
    pub fn is_end_of_chain(&self, cluster: u32) -> bool { 
        if cluster as usize >= self.entries.len() { // usize pour pouvoir l'utiliser en index
            return true;
        }
        let entry = self.entries[cluster as usize];
        entry >= 0x0FFFFFF8
    }
    
    /// Check if cluster is bad
    pub fn is_bad_cluster(&self, cluster: u32) -> bool {
        if cluster as usize >= self.entries.len() {
            return true;
        }
        self.entries[cluster as usize] == 0x0FFFFFF7
    }
    
    /// cluster = free ?
    pub fn is_free_cluster(&self, cluster: u32) -> bool {
        if cluster as usize >= self.entries.len() {
            return false;
        }
        self.entries[cluster as usize] == 0
    }
    
    /// nb entrée en FAT
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

