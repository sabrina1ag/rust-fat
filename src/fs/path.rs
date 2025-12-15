use alloc::string::String;
use alloc::vec::Vec;

/// Path parsing error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// Invalid path format
    InvalidFormat(String),
    /// Empty path
    Empty,
    /// Path component too long
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

/// Path representation for FAT32 filesystem
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    /// Path components (without separators)
    components: Vec<String>,
    /// Whether this is an absolute path (starts with /)
    absolute: bool,
}

impl Path {
    /// Create a new path from a string
    pub fn new(path_str: &str) -> Result<Self, PathError> {
        if path_str.is_empty() {
            return Err(PathError::Empty);
        }
        
        let absolute = path_str.starts_with('/');
        let path_str = if absolute { &path_str[1..] } else { path_str };
        
        let components: Vec<String> = if path_str.is_empty() {
            Vec::new()
        } else {
            path_str
                .split('/')
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
        
        // Validate components
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
    
    /// Create root path
    pub fn root() -> Self {
        Self {
            components: Vec::new(),
            absolute: true,
        }
    }
    
    /// Check if path is absolute
    pub fn is_absolute(&self) -> bool {
        self.absolute
    }
    
    /// Check if path is root
    pub fn is_root(&self) -> bool {
        self.absolute && self.components.is_empty()
    }
    
    /// Get path components
    pub fn components(&self) -> &[String] {
        &self.components
    }
    
    /// Join with another path
    pub fn join(&self, other: &Path) -> Result<Self, PathError> {
        if other.is_absolute() {
            return Ok(other.clone());
        }
        
        let mut new_components = self.components.clone();
        new_components.extend_from_slice(other.components());
        
        // Resolve ".." components
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
    
    /// Get parent path
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
    
    /// Get file name (last component)
    pub fn file_name(&self) -> Option<&String> {
        self.components.last()
    }
    
    /// Convert to string representation
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

/// Owned path buffer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathBuf {
    path: Path,
}

impl PathBuf {
    /// Create a new PathBuf from a string
    pub fn new(path_str: &str) -> Result<Self, PathError> {
        Ok(Self {
            path: Path::new(path_str)?,
        })
    }
    
    /// Create root PathBuf
    pub fn root() -> Self {
        Self {
            path: Path::root(),
        }
    }
    
    /// Get path reference
    pub fn as_path(&self) -> &Path {
        &self.path
    }
    
    /// Push a component onto the path
    pub fn push(&mut self, component: &str) -> Result<(), PathError> {
        let new_path = Path::new(component)?;
        self.path = self.path.join(&new_path)?;
        Ok(())
    }
    
    /// Pop the last component
    pub fn pop(&mut self) -> bool {
        if let Some(parent) = self.path.parent() {
            self.path = parent;
            true
        } else {
            false
        }
    }
    
    /// Convert to string
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
