use std::path::PathBuf;
use crate::conf::PathError;

pub struct SSHPathBindings {
    pub is_canonical: Box<dyn Fn(&str) -> Result<bool, PathError>>,
    pub absolute: Box<dyn Fn(&str) -> Result<PathBuf, PathError>>,
    pub canonical: Box<dyn Fn(&str) -> Result<PathBuf, PathError>>,
    pub to_remote: Box<dyn Fn(&str) -> PathBuf>
}
