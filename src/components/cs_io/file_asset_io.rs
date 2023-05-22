
use bevy_utils::BoxedFuture;
use std::fs::File;
use std::{
    io::Read,
    path::{Path, PathBuf},
};

use super::{AssetIoError, AssetIo};

pub struct FileAssetIo {
    root_path: PathBuf
}

impl FileAssetIo {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileAssetIo {
            root_path: PathBuf::new().join(path.as_ref()),
        }
    }
}

impl AssetIo for FileAssetIo {
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            let full_path = self.root_path.join(path);
            match File::open(&full_path) {
                Ok(mut file) => {
                    file.read_to_end(&mut bytes)?;
                }
                Err(e) => {
                    return if e.kind() == std::io::ErrorKind::NotFound {
                        Err(AssetIoError::NotFound(full_path))
                    } else {
                        Err(e.into())
                    }
                }
            }
            Ok(bytes)
        })
    }


}
