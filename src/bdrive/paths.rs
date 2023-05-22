use std::path::PathBuf;
use crate::bdrive::BDrive;
use crate::conf::PathError;
use crate::fs::File;
use crate::fs::state::*;


impl BDrive {
    pub fn load_local_file(&self, path: &str) -> Result<File<Local>, PathError> {
        let mut can = self.path_key.clone();
        can.push(path);
        if self.paths.is_canonical(can.to_str().unwrap())? {
            Ok(File::from(can))
        } else {
            Err(PathError::Outbound(PathBuf::from(path)))
        }
    }
}