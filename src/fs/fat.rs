use crate::fs::{FileSystem, FileSystemError, DirEntry};
use crate::fs::boot::BootSector; // on utilise direct BootSector au lieu du chemin fs/boot
use crate::fs::fat_table::FatTable; 
use crate::fs::cluster::ClusterChain;
use crate::fs::directory::Directory;
use crate::fs::path::{Path, PathBuf};
use alloc::vec::Vec; // on fait des allocs vu qu'on est en no_std, si on etait en std on aurait ecrit std::vec::Vec et le alloc est implicite
use alloc::string::String;

/// CE FICHIER EST HORRIBLE
pub struct Fat32Fs {
    /// Boot sector
    boot_sector: BootSector,
    /// FAT table (tableau avec num cluster et son suiv genre (5:6 ou 5: fin ou 5: erreur))
    fat_table: FatTable,
    /// dossier ou l'on est genre quand je crée des dossiers dans la fat ? ou bien autre chose ?
    current_path: PathBuf,
    /// device_data contenu complet de la fat32
    device_data: Vec<u8>,
}

impl Fat32Fs { //bloc de fonctions et methodes associés a fat32Fs

    //fonction de creation d'une instance de FAT32
    pub unsafe fn new(device_data: &[u8]) -> Result<Self, FileSystemError> { //on retourne la structure ou une erreur

        let boot_sector = BootSector::from_bytes(device_data)?; //lire les 512 premier octet de device data et remplir boot sector
        
        // offset debut fat et taille fat, à partir de boot sector, on multiplie pour avoir la taille en octet
        let fat_start = boot_sector.fat_start_sector() * boot_sector.bytes_per_sector();
        let fat_size = boot_sector.sectors_per_fat() * boot_sector.bytes_per_sector();
        
        // si la taille de la fat est plus grande erreur
        if (fat_start as usize + fat_size as usize) > device_data.len() {
            return Err(FileSystemError::InvalidFat("FAT table out of bounds".into()));
        }
        
        let fat_data = &device_data[fat_start as usize..(fat_start as usize + fat_size as usize)];
        //fat_data contient exactement tout les octet de device_data
        let fat_table = FatTable::from_bytes(fat_data)?;
        // fat_table contient le num du cluster et son contenu / code erreur / code fin et ? erreur si Fat invalide


        Ok(Self { // tout est good on a notre structure de FAT32
            boot_sector,
            fat_table,
            current_path: PathBuf::root(),
            device_data: device_data.to_vec(),
        })
    }
    
    /// retourner la chaine complente d'un cluster a partir d'un cluster i (start_cluster)
    pub fn get_cluster_chain(&self, start_cluster: u32) -> Result<ClusterChain, FileSystemError> {
        ClusterChain::new(&self.fat_table, start_cluster)
    }
    //ClusterChain c'est un constructeur on lui donne la fat table et le start cluster
    
    ///lire le contenu d'un cluster 
    pub fn read_cluster(&self, cluster: u32) -> Result<Vec<u8>, FileSystemError> {
        let cluster_size = self.boot_sector.cluster_size() as usize; 
        let data_start = self.boot_sector.data_start_sector() * self.boot_sector.bytes_per_sector(); //offset en octet de la zone data_start
        let cluster_offset = ((cluster - 2) * self.boot_sector.sectors_per_cluster()) 
            * self.boot_sector.bytes_per_sector();  // chaque cluster commence a partir de 2 et on multiplie pour avoir l'offset
        let offset = (data_start + cluster_offset) as usize;
        
        if offset + cluster_size > self.device_data.len() { //offset superieur à l'image ERREUR
            return Err(FileSystemError::IoError("Cluster out of bounds".into()));
        }
        
        Ok(self.device_data[offset..offset + cluster_size].to_vec()) //retourne vecteur d'octet (indexation cluster)
    }
    
    // j'ai un chemin d'acces et je veux trouver le cluster correspondant.
    fn get_directory_cluster(&self, path: &Path) -> Result<u32, FileSystemError> { //pk fonction privé ?
        if path.is_root() { //si c'est la racine ya rien à faire
            return Ok(self.boot_sector.root_cluster());
        }
        
        // se positionner sur le dossier racine avant de boucler
        let mut current_cluster = self.boot_sector.root_cluster();

        // on parcourt chaque element du chemin
        for component in path.components() {
            // lire contenu dossier courant
            let chain = self.get_cluster_chain(current_cluster)?;
            //charger tout le contenu du dossier dans un tampon
            let mut directory_data = Vec::new(); //pk un tampon ?
            for &cluster_num in chain.clusters() {
                let cluster_data = self.read_cluster(cluster_num)?;
                directory_data.extend_from_slice(&cluster_data);
            }
            
            let entry = Directory::find_entry(&directory_data, component)? //chercher dans le dossier courant un sous dossier avec le nom dans component
                .ok_or_else(|| {
                    let mut msg = String::from("Directory not found: ");
                    msg.push_str(component);
                    FileSystemError::DirectoryNotFound(msg)
                })?;
            
            if !entry.is_directory() { // si entrée trouvé mais pas un directory
                let mut msg = String::from("Not a directory: ");
                msg.push_str(component); //afficher que le dosiser en entrée n'est pas un directory
                return Err(FileSystemError::DirectoryNotFound(msg));
            }
            
            current_cluster = entry.first_cluster(); // passer au cluster suivant
            if current_cluster == 0 { //cluster de fin 
                return Err(FileSystemError::DirectoryNotFound(
                    "Invalid directory cluster".into()
                ));
            }
        }
        
        Ok(current_cluster)
    }
    
    /// Get boot sector reference
    pub fn boot_sector(&self) -> &BootSector {
        &self.boot_sector
    }
}

impl FileSystem for Fat32Fs {
    /// fonction qui liste les fichiers dossiers dans un chemin
    fn list(&self, path: &str) -> Result<Vec<DirEntry>, FileSystemError> {
        let target_path = if path.starts_with('/') { //chemin absolu
            Path::new(path)?
        } else {
            self.current_path.as_path().join(&Path::new(path)?)? //chemin relatif on le concatene au chemin courant
        };
        
        // retrouver le cluster
        let dir_cluster = self.get_directory_cluster(&target_path)?;
        
        // retrouver la chaine a partir du premier cluster genre [5, 6, 7]
        let chain = self.get_cluster_chain(dir_cluster)?;
        
        // mettre tout le contenu dans directory_data
        let mut directory_data = Vec::new();
        for &cluster_num in chain.clusters() {
            let cluster_data = self.read_cluster(cluster_num)?;
            directory_data.extend_from_slice(&cluster_data);
        }
        
        // Parse entries
        unsafe {
            Directory::read_entries(&chain, &directory_data) //convertir en structure directory (qui represente un dossier)
        }
    }
    
    /// lire entierement un fichier
    fn read_file(&self, path: &str) -> Result<Vec<u8>, FileSystemError> {
        let target_path = if path.starts_with('/') {
            Path::new(path)?
        } else {
            self.current_path.as_path().join(&Path::new(path)?)?
        };
        
        // Get file name
        let file_name = target_path.file_name()
            .ok_or_else(|| {
                let path_str = target_path.to_string();
                FileSystemError::FileNotFound(path_str)
            })?;
        
        // Get parent directory
        let parent_path = target_path.parent()
            .ok_or_else(|| FileSystemError::DirectoryNotFound("Root directory".into()))?;
        
        // Get parent directory cluster
        let parent_cluster = self.get_directory_cluster(&parent_path)?;
        
        // Read parent directory
        let chain = self.get_cluster_chain(parent_cluster)?;
        let mut directory_data = Vec::new();
        for &cluster_num in chain.clusters() {
            let cluster_data = self.read_cluster(cluster_num)?;
            directory_data.extend_from_slice(&cluster_data);
        }
        
        // Find file entry
        let path_str = target_path.to_string();
        let entry = Directory::find_entry(&directory_data, file_name)?
            .ok_or_else(|| FileSystemError::FileNotFound(path_str.clone()))?;
        
        if !entry.is_file() {
            let mut msg = path_str;
            msg.push_str(" is not a file");
            return Err(FileSystemError::FileNotFound(msg));
        }
        
        // Get first cluster
        let first_cluster = entry.first_cluster();
        if first_cluster == 0 {
            return Ok(Vec::new());
        }
        
        // Get cluster chain
        let chain = self.get_cluster_chain(first_cluster)?;
        
        // Read all file data
        let mut file_data = Vec::new();
        for &cluster_num in chain.clusters() {
            let cluster_data = self.read_cluster(cluster_num)?;
            file_data.extend_from_slice(&cluster_data);
        }
        
        // Truncate to file size
        let file_size = entry.file_size() as usize;
        if file_data.len() > file_size {
            file_data.truncate(file_size);
        }
        
        Ok(file_data)
    }
    
    /// Change current directory
    fn cd(&mut self, path: &str) -> Result<(), FileSystemError> {
        let target_path = if path.starts_with('/') {
            Path::new(path)?
        } else {
            self.current_path.as_path().join(&Path::new(path)?)?
        };
        
        if target_path.is_root() {
            self.current_path = PathBuf::from(target_path);
            return Ok(());
        }
        
        // Verify directory exists
        let dir_cluster = self.get_directory_cluster(&target_path)?;
        let chain = self.get_cluster_chain(dir_cluster)?;
        let mut directory_data = Vec::new();
        for &cluster_num in chain.clusters() {
            let cluster_data = self.read_cluster(cluster_num)?;
            directory_data.extend_from_slice(&cluster_data);
        }
        
        // Get directory name
        let path_str = target_path.to_string();
        let dir_name = target_path.file_name()
            .ok_or_else(|| FileSystemError::DirectoryNotFound(path_str.clone()))?;
        
        // Get parent directory
        let parent_path = target_path.parent()
            .ok_or_else(|| FileSystemError::DirectoryNotFound("Root directory".into()))?;
        
        let parent_cluster = self.get_directory_cluster(&parent_path)?;
        let parent_chain = self.get_cluster_chain(parent_cluster)?;
        let mut parent_data = Vec::new();
        for &cluster_num in parent_chain.clusters() {
            let cluster_data = self.read_cluster(cluster_num)?;
            parent_data.extend_from_slice(&cluster_data);
        }
        
        let entry = Directory::find_entry(&parent_data, dir_name)?
            .ok_or_else(|| FileSystemError::DirectoryNotFound(path_str.clone()))?;
        
        if !entry.is_directory() {
            let mut msg = path_str;
            msg.push_str(" is not a directory");
            return Err(FileSystemError::DirectoryNotFound(msg));
        }
        
        // Update current path
        self.current_path = PathBuf::from(target_path);
        
        Ok(())
    }
    
    /// Get current directory path
    fn pwd(&self) -> String {
        self.current_path.to_string()
    }
    
    /// Create a new file at the given path
    fn create_file(&mut self, path: &str) -> Result<(), FileSystemError> {
        // Note: Full implementation requires FAT modification
        // This is a placeholder that validates the path
        let _target_path = if path.starts_with('/') {
            Path::new(path)?
        } else {
            self.current_path.as_path().join(&Path::new(path)?)?
        };
        
        Err(FileSystemError::Unsupported(
            "File creation requires FAT modification which is not yet implemented".into()
        ))
    }
    
    /// Write data to a file at the given path
    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), FileSystemError> {
        // Note: Full implementation requires FAT and cluster modification
        let _target_path = if path.starts_with('/') {
            Path::new(path)?
        } else {
            self.current_path.as_path().join(&Path::new(path)?)?
        };
        
        let _ = data; // Suppress unused warning
        Err(FileSystemError::Unsupported(
            "File writing requires FAT and cluster modification which is not yet implemented".into()
        ))
    }
}
