use crate::fs::FileSystemError;
use alloc::string::String;
use alloc::vec::Vec;

/// entrée de repertoire fat32 format short name 8+3
#[repr(C, packed)]
pub struct DirectoryEntry {
    /// nom court du fichier 8 caracteres + 3 pour le format
    pub name: [u8; 11],
    pub attributes: u8,
    /// Champ réservé (NT, généralement ignoré)
    pub nt_reserved: u8,
    /// Creation time (tenths of second)
    pub creation_time_tenths: u8,
    /// Heure creation
    pub creation_time: u16,
    /// date creation
    pub creation_date: u16,
    /// dernier acces en date
    pub last_access_date: u16,
    /// Partie haute du cluster (16-31)
    pub first_cluster_high: u16,
    /// Heure de dernière écriture
    pub last_write_time: u16,
     /// Date de dernière écriture
    pub last_write_date: u16,
    /// Partie basse du cluster (bits 0-15)
    pub first_cluster_low: u16,
     /// Taille du fichier en octets
    pub file_size: u32,
}

impl DirectoryEntry {
    /// /// Crée une DirectoryEntry à partir de 32 octets bruts lus sur le disque
    /// 
    /// # Documentation d'une fonction Unsafe
    /// 
    /// On lit directement la mémoire comme une structure FAT32.
    /// C’est safe uniquement si on est sûr que les données font bien 32 octets
    /// et qu’elles viennent d’un vrai filesystem FAT.
    pub unsafe fn from_bytes(data: &[u8]) -> Result<Self, FileSystemError> {
        if data.len() < 32 {
            return Err(FileSystemError::DirectoryEntryError(
                "Directory entry must be at least 32 bytes".into()
            ));
        }
        
        // la taille correspond on procede a la lecture depuis la memoire
        let entry = core::ptr::read(data.as_ptr() as *const DirectoryEntry);
        
        // (0x00) = entrée libre ou (0xE5) = entrée supprimé
        if entry.name[0] == 0x00 || entry.name[0] == 0xE5 {
            return Err(FileSystemError::DirectoryEntryError(
                "Empty or deleted entry".into()
            ));
        }
        
        Ok(entry)
    }
    C
    /// verifier si c'est un dossier
    pub fn is_directory(&self) -> bool {
        (self.attributes & 0x10) != 0
    }
    
    /// verifier si c'est un fichier classique
    pub fn is_file(&self) -> bool {
        !self.is_directory() && (self.attributes & 0x08) == 0
    }
    
    /// verifier si l'entrée est un label de volume
    pub fn is_volume_label(&self) -> bool {
        (self.attributes & 0x08) != 0
    }
    
    /// reconstruire le cluster concanterner high | low
    pub fn first_cluster(&self) -> u32 {
        ((self.first_cluster_high as u32) << 16) | (self.first_cluster_low as u32)
    }
    
    /// retourner taille fichier
    pub fn file_size(&self) -> u32 {
        self.file_size
    }
    
    /// convertir le nom court fat, en string lisible
    pub fn short_name(&self) -> Result<String, FileSystemError> {
        let mut name_bytes = Vec::new();
        
        // lecture 'nom'
        let name_part = &self.name[0..8];
        for &b in name_part.iter() {
            if b == 0x20 {
                break;
            }
            if b != 0x20 {
                name_bytes.push(b);
            }
        }
        
        // lecture .extension
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
        // si une extension existe on la concatene manuellement
        if !ext_bytes.is_empty() {
            let ext_str = String::from(
                core::str::from_utf8(&ext_bytes)
                    .map_err(|_| FileSystemError::DirectoryEntryError("Invalid UTF-8 in extension".into()))?
            );
            // concatenation manuelle en no_std
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

/// utilisée pour les noms longs FAT32
#[repr(C, packed)]
pub struct LongFileNameEntry {
    /// Numero de sequence + flag
    pub sequence: u8,
    /// caractere  1-5 du nom
    pub name1: [u16; 5],
    pub attributes: u8,
    
    pub type_: u8,
    
    pub checksum: u8,
    /// les caracteres 6-11 du nom
    pub name2: [u16; 6],
    /// First cluster (must be 0x0000)
    pub first_cluster: u16,
    /// les caracteres 12-13 du nom
    pub name3: [u16; 2],
}

impl LongFileNameEntry {
    /// Vérifie que l’entrée LFN est valide
    pub fn is_valid(&self) -> bool {
        self.attributes == 0x0F && self.type_ == 0x00 && self.first_cluster == 0x0000
    }
    
     /// Retourne le numéro de séquence (sans les flags)
    pub fn sequence_number(&self) -> u8 {
        self.sequence & 0x3F
    }
    
    /// Indique si c’est la dernière entrée LFN
    pub fn is_last(&self) -> bool {
        (self.sequence & 0x40) != 0
    }
    
     /// Récupère tous les caractères UTF-16 de cette entrée
    pub fn name_chars(&self) -> Vec<u16> {
        let mut chars = Vec::new();
        // Copie locale pour éviter les problèmes d’alignement (packed)
        let name1 = self.name1;
        let name2 = self.name2;
        let name3 = self.name3;
        chars.extend_from_slice(&name1);
        chars.extend_from_slice(&name2);
        chars.extend_from_slice(&name3);
        chars
    }
}

// Entrée de dossier finale utilisée par le FS
pub struct DirEntry {
    /// entrée FAT Standard
    pub entry: DirectoryEntry,
    /// Nom Long si present
    pub long_name: Option<String>,
}

impl DirEntry {
    
    pub fn new(entry: DirectoryEntry) -> Self {
        Self {
            entry,
            long_name: None,
        }
    }
    
    /// Ajouter un nom long a l'entrée
    pub fn with_long_name(mut self, long_name: String) -> Self {
        self.long_name = Some(long_name);
        self
    }
    
    /// Nom à afficher (long name si dispo, sinon short name)
    pub fn name(&self) -> Result<String, FileSystemError> {
        if let Some(ref long_name) = self.long_name {
            Ok(long_name.clone())
        } else {
            self.entry.short_name()
        }
    }
    
    /// dossier ?
    pub fn is_directory(&self) -> bool {
        self.entry.is_directory()
    }
    
    /// fichier ?
    pub fn is_file(&self) -> bool {
        self.entry.is_file()
    }
    
    /// Cluster de depart
    pub fn first_cluster(&self) -> u32 {
        self.entry.first_cluster()
    }
    
    /// taille du fichier
    pub fn file_size(&self) -> u32 {
        self.entry.file_size()
    }
}
