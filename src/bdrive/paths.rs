use walkdir::WalkDir;
use crate::bdrive::BDrive;
use crate::conf::PathError;
use crate::fs::File;
use crate::fs::state::*;

#[derive(Debug)]
pub enum PathSpecial {
    Outbound(String),
    Ignored(String),
    Malformed(String),
    Unknown
}

impl BDrive {
    fn validate_path<T>(&self, path: String, wrapper: fn(&str) -> T) -> Result<T, PathSpecial> {
        if self.paths.is_canonical(&path)? {
            Ok(wrapper(&path))
        } else {
            Err(PathSpecial::Outbound(path))
        }
    }

    fn canonicalize(&self, path: &str) -> String {
        match path.strip_prefix('/') {
            Some(strip) => strip.to_string(),
            None => {
                [self.exe_path.to_str().unwrap(), path].join("/")
            }
        }
    }

    pub fn load_file(&self, path: &str) -> Result<File<Local>, PathSpecial> {
        self.validate_path(self.canonicalize(path), |f| File::from(f))
    }

    pub fn scan_dir(&self, path: &str) -> Result<Vec<Result<File<Local>, PathSpecial>>, PathError> {
        let pc = self.canonicalize(path);
        self.paths.is_canonical(&pc)?;
        Ok(WalkDir::new(self.canonicalize(path))
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|f| {
                self.validate_path(f.path().to_str().unwrap().to_string(), |f| File::from(f))
            })
            .collect::<Vec<Result<File<Local>, PathSpecial>>>())
    }
}

impl From<PathError> for PathSpecial {
    fn from(value: PathError) -> Self {
        match value {
            PathError::Outbound(s) => PathSpecial::Outbound(s.to_str().unwrap().to_string()),
            PathError::Malformed(s) => PathSpecial::Malformed(s),
            // enable this for future `PathError`s
            #[allow(unreachable_patterns)]
            _ => PathSpecial::Unknown
        }
    }
}
