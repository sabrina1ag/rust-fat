use alloc::string::String;
use alloc::vec::Vec;

/// Erreurs Possibles lors du parsing d'un chemin
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// chemin invalide
    InvalidFormat(String),
    /// chemon vide
    Empty,
    /// nom de fichiers / dossiers trop long > 255 caracteres
    ComponentTooLong,
}

impl core::fmt::Display for PathError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PathError::InvalidFormat(msg) => write!(f, "Invalid path format: {}", msg),
            PathError::Empty => write!(f, "Empty path"),
            PathError::ComponentTooLong => write!(f, "Path component too long"),
        }
    }
}

/// Représentation interne d’un chemin FAT32
// Pour : 
/// "/home/user/docs"
/// -> absolute = true pck ya /
/// -> components = ["home", "user", "docs"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    /// les compostants du chemin 
    /// /home/user -> ["home", "user"]
    components: Vec<String>,
    /// indique si le chemin commence par /
    absolute: bool,
}

impl Path {
    /// creer un path à partir d'une string
    pub fn new(path_str: &str) -> Result<Self, PathError> {
        if path_str.is_empty() {
            return Err(PathError::Empty);
        }
        // chemin absolu ?
        let absolute = path_str.starts_with('/');
        let path_str = if absolute { &path_str[1..] } else { path_str };
        
        let components: Vec<String> = if path_str.is_empty() {
            Vec::new()
        } else {
            path_str
                .split('/')
                // "home/./docs" → ["home", "docs"]
                .filter(|s| !s.is_empty() && *s != ".")
                .map(|s| {
                    if s == ".." {
                        return String::from("..");
                    }
                    if s.len() > 255 {
                        return String::new();
                    }
                    String::from(s)
                })
                .collect()
        };
        
        // Vérification finale des composants
        for component in &components {
            if component.is_empty() {
                return Err(PathError::InvalidFormat("Empty component".into()));
            }
            if component.len() > 255 {
                return Err(PathError::ComponentTooLong);
            }
        }
        
        Ok(Self {
            components,
            absolute,
        })
    }
    
    /// Creer chemin racine
    pub fn root() -> Self {
        Self {
            components: Vec::new(),
            absolute: true,
        }
    }
    
    /// /test = true , test = false
    pub fn is_absolute(&self) -> bool {
        self.absolute
    }
    
    /// Check if path is root
    pub fn is_root(&self) -> bool {
        self.absolute && self.components.is_empty()
    }
    
    /// /test/a donne ["test","a"]
    pub fn components(&self) -> &[String] {
        &self.components
    }
    
    /// joindre deux chemins actuelle + en input
    // actuelle /test other = docs/test.txt
    // retoutne /test/docs/test.txt
    pub fn join(&self, other: &Path) -> Result<Self, PathError> {
        if other.is_absolute() {
            return Ok(other.clone());
        }
        
        let mut new_components = self.components.clone();
        new_components.extend_from_slice(other.components());
        
        // traiter les ..
        // donc si test,user,..,source on aura en sortie test, source
        let mut resolved = Vec::new();
        for component in new_components {
            if component == ".." {
                if !resolved.is_empty() {
                    resolved.pop();
                }
            } else {
                resolved.push(component);
            }
        }
        
        Ok(Self {
            components: resolved,
            absolute: self.absolute,
        })
    }
    
    /// un dossier en arriere
    pub fn parent(&self) -> Option<Self> {
        if self.components.is_empty() {
            return None;
        }
        
        let mut parent_components = self.components.clone();
        parent_components.pop();
        
        Some(Self {
            components: parent_components,
            absolute: self.absolute,
        })
    }
    
    /// derniere element = nom fichier
    pub fn file_name(&self) -> Option<&String> {
        self.components.last()
    }
    
    /// Reconstruire le chemin en string
    pub fn to_string(&self) -> String {
        let mut result = String::new();
        if self.absolute {
            result.push('/');
        }
        for (i, component) in self.components.iter().enumerate() {
            if i > 0 {
                result.push('/');
            }
            result.push_str(component);
        }
        result
    }
}

/// Version mutable du path pour cd et pwd
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathBuf {
    path: Path,
}

impl PathBuf {
     /// ex: PathBuf::new("/home/user")
    pub fn new(path_str: &str) -> Result<Self, PathError> {
        Ok(Self {
            path: Path::new(path_str)?,
        })
    }
    
    /// PathBuf representant "/"
    pub fn root() -> Self {
        Self {
            path: Path::root(),
        }
    }
    
    /// Acces en lecture au path
    pub fn as_path(&self) -> &Path {
        &self.path
    }
    
    /// push("docs") sur "/home/user" → "/home/user/docs"
    pub fn push(&mut self, component: &str) -> Result<(), PathError> {
        let new_path = Path::new(component)?;
        self.path = self.path.join(&new_path)?;
        Ok(())
    }
    
    /// "/home/user/docs" → "/home/user"
    pub fn pop(&mut self) -> bool {
        if let Some(parent) = self.path.parent() {
            self.path = parent;
            true
        } else {
            false
        }
    }
    
    /// Convertit en string
    pub fn to_string(&self) -> String {
        self.path.to_string()
    }
}

impl From<Path> for PathBuf {
    fn from(path: Path) -> Self {
        Self { path }
    }
}

impl core::fmt::Display for Path {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl core::fmt::Display for PathBuf {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.path.to_string())
    }
}
