use image::ImageReader;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tar::Builder;
use zstd::stream::write::Encoder;

use crate::Config;

pub struct Packager {
    config: Config,
    path: PathBuf,
}

impl Packager {
    pub fn new(conf: Config, path: impl AsRef<Path>) -> Self {
        Packager {
            config: conf,
            path: path.as_ref().to_owned(),
        }
    }

    pub fn pack<T: AsRef<Path>>(&self, out: T) -> io::Result<()> {
        let out_path = out.as_ref();

        // Paso 1: Validar que self.path existe y es un directorio
        if !self.path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Path does not exist: {}", self.path.display()),
            ));
        }
        if !self.path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Path is not a directory: {}", self.path.display()),
            ));
        }

        // Validar que self.path/images existe
        let images_dir = self.path.join("images");
        if !images_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Images directory does not exist: {}", images_dir.display()),
            ));
        }
        if !images_dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Images path is not a directory: {}", images_dir.display()),
            ));
        }

        // Paso 2: Serializar configuración
        let manifest_bytes = toml::to_string(&self.config)
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("TOML serialization failed: {}", e),
                )
            })?
            .into_bytes();

        // Paso 3: Crear archivo tar en memoria
        let mut tar_data = Vec::new();
        {
            let mut tar_builder = Builder::new(&mut tar_data);

            // Añadir manifest.toml
            let mut header = tar::Header::new_gnu();
            header.set_size(manifest_bytes.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar_builder.append_data(&mut header, "manifest.toml", &manifest_bytes[..])?;

            // Paso 4: Añadir imágenes válidas
            for entry in fs::read_dir(&images_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if is_image(&path)? {
                        let file_name = path.file_name().unwrap().to_string_lossy();
                        let entry_path = format!("images/{}", file_name);
                        tar_builder.append_path_with_name(&path, entry_path)?;
                    }
                }
            }
        }

        // Paso 5: Comprimir tar con zstd
        let out_file = File::create(out_path)?;
        let mut encoder = Encoder::new(out_file, 3)?; // nivel de compresión 3
        encoder.write_all(&tar_data)?;
        encoder.finish()?;

        Ok(())
    }
}

// Paso 5: Función auxiliar para validar imágenes
fn is_image(path: &Path) -> io::Result<bool> {
    match ImageReader::open(path) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
