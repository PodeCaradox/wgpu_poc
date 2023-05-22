#[cfg(not(target_arch = "wasm32"))]
pub mod file_asset_io;
#[cfg(target_arch = "wasm32")]
pub mod wasm_asset_io;


use downcast_rs::{impl_downcast, Downcast};
#[cfg(not(target_arch = "wasm32"))]
pub use file_asset_io::*;
#[cfg(target_arch = "wasm32")]
pub use wasm_asset_io::*;



use std::{
    io,
    path::{Path, PathBuf},
};
use bevy_utils::BoxedFuture;
use thiserror::Error;

/// Errors that occur while loading assets.
#[derive(Error, Debug)]
pub enum AssetIoError {
    /// Path not found.
    #[error("path not found: {0}")]
    NotFound(PathBuf),

    /// Encountered an I/O error while loading an asset.
    #[error("encountered an io error while loading asset: {0}")]
    Io(#[from] io::Error),

}

pub trait AssetIo: Downcast + Send + Sync + 'static {
    /// Returns a future to load the full file data at the provided path.
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>>;
}

impl_downcast!(AssetIo);

pub fn get_asset_store() -> Box<dyn AssetIo>{
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let asset_io: Box<dyn AssetIo> = Box::new(WasmAssetIo::new(""));
        }else{
            let asset_io: Box<dyn AssetIo> = Box::new(FileAssetIo::new(""));
        }
    }
    asset_io
}