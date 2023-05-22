use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::path::{Path, PathBuf};
use crate::conf::PathsConf;

#[derive(Debug)]
pub enum PathError {
    Outbound(PathBuf),
    Malformed(String)
}

impl Display for PathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for PathError {}

impl PathsConf {
    pub fn is_canonical(&self, rel: &str) -> Result<bool, PathError> {
        let p = PathBuf::from(rel);
        p.canonicalize().map(|f| f.starts_with(&self.local)).map_err(|e| PathError::Malformed(rel.to_string()))
    }

    pub fn absolute(&self, rel: &str) -> Result<PathBuf, PathError> {
        let p = PathBuf::from(rel);
        match p.canonicalize() {
            Ok(p) => if p.starts_with(&self.local) {
                Ok(p)
            } else {
                Err(PathError::Outbound(p))
            }
            Err(_) => Err(PathError::Malformed(rel.to_string()))
        }
    }

    pub fn canonical(&self, rel: &str) -> Result<PathBuf, PathError> {
        Ok(PathBuf::from(self.absolute(rel)?.strip_prefix(&self.local).unwrap()))
    }

    pub fn to_remote(&self, rel: &str) -> PathBuf {
        let p = Path::new(&rel);
        p.join(Path::new(&self.remote))
    }
}
