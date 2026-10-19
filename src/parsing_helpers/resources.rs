use std::io::{Cursor, Read};
use std::marker::PhantomData;
use bytes::Bytes;
use serde::{Deserialize, Deserializer};
use zip::ZipArchive;
use crate::parsing_helpers::resources::ResourceError::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Resource<T: ResourceType>(pub(crate) String, PhantomData<T>);

impl <'de, T: ResourceType> Deserialize<'de> for Resource<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Ok(Self(s, PhantomData))
    }
}

impl<T: ResourceType> Resource<T> {
    pub fn get_file(&self, archive: &mut ZipArchive<Cursor<Bytes>>) -> Result<Vec<Vec<u8>>, ResourceError> {
        T::get_file(&*self.0, archive)
    }
}

pub trait ResourceType {
    fn get_file(resource_name: &str, archive: &mut ZipArchive<Cursor<Bytes>>) -> Result<Vec<Vec<u8>>, ResourceError>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RootPNG;
impl ResourceType for RootPNG {
    fn get_file(resource_name: &str, archive: &mut ZipArchive<Cursor<Bytes>>) -> Result<Vec<Vec<u8>>, ResourceError> {
        let candidates = [
            format!("{}.png", resource_name),
            format!("{}.svg", resource_name),
        ];

        get_archived_files(archive, &candidates)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WheelPNG;
impl ResourceType for WheelPNG {
    fn get_file(resource_name: &str, archive: &mut ZipArchive<Cursor<Bytes>>) -> Result<Vec<Vec<u8>>, ResourceError> {
        let candidates = [
            format!("wheels/{}.png", resource_name),
            format!("{}.png", resource_name),
        ];

        get_archived_files(archive, &candidates)

    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Model3DFile;
impl ResourceType for Model3DFile {
    fn get_file(resource_name: &str, archive: &mut ZipArchive<Cursor<Bytes>>) -> Result<Vec<Vec<u8>>, ResourceError> {
        let candidates = [
            format!("models/3ds/{}.3ds", resource_name),
            format!("models/3ds_low/{}.3ds", resource_name),
            format!("models/3ds_high/{}.3ds", resource_name),
            format!("models/gltf/{}.glb", resource_name),
            format!("models/gltf_low/{}.glb", resource_name),
            format!("models/gltf_high/{}.glb", resource_name),
        ];

        get_archived_files(archive, &candidates)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureFile;
impl ResourceType for TextureFile {
    fn get_file(resource_name: &str, archive: &mut ZipArchive<Cursor<Bytes>>) -> Result<Vec<Vec<u8>>, ResourceError> {
        let candidates = [
            format!("textures/{}.png", resource_name),
            format!("textures/{}.jpg", resource_name),
            format!("{}.png", resource_name),
            format!("{}.jpg", resource_name),
        ];

        get_archived_files(archive, &candidates)
    }
}

fn get_archived_files(archive: &mut ZipArchive<Cursor<Bytes>>, candidates: &[String])
                      -> Result<Vec<Vec<u8>>, ResourceError>
{
    let mut files = Vec::new();

    for path in candidates {
        match archive.by_name(path) {
            Ok(mut archive_file) => {
                let mut data = Vec::new();

                archive_file
                    .read_to_end(&mut data)
                    .map_err(|e| UnzipError(e.to_string()))?;
                files.push(data);
            }
            Err(_) => {}
        }
    }
    if files.is_empty() {
        Err(NoFileFoundError)
    } else {
        Ok(files)
    }
}

pub enum ResourceError {
    UnzipError(String),
    NoFileFoundError,
}