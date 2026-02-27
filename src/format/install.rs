use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

use tar::Archive;
use zstd::Decoder;

use crate::{Config, decompresion_folder};

pub struct PackInstaller {
    file_path: PathBuf,
    pack_name: String,
    dest_dir: PathBuf,
}

impl PackInstaller {
    pub fn new<T: AsRef<Path>>(file: T) -> Self {
        Self {
            file_path: file.as_ref().to_path_buf(),
            pack_name: "unknown".to_string(),
            dest_dir: decompresion_folder(),
        }
    }

    pub fn install(&mut self) -> io::Result<()> {
        self.read_manifest()?;
        self.create_dest_dir()?;
        self.unpack_archive()?;
        Ok(())
    }

    fn read_manifest(&mut self) -> io::Result<()> {
        let bin_file = File::open(&self.file_path)?;
        let decoder = Decoder::new(bin_file)?;
        let mut archive = Archive::new(decoder);

        // Default name from filename
        self.pack_name = self
            .file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;

            if path
                .file_name()
                .map(|n| n == "manifest.toml")
                .unwrap_or(false)
            {
                let mut contents = String::new();
                entry.read_to_string(&mut contents)?;

                match toml::from_str::<Config>(&contents) {
                    Ok(config) => {
                        if let Some(name) = config.name {
                            self.pack_name = sanitize_name(&name);
                        }
                    }
                    Err(e) => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Failed to parse manifest.toml: {}", e),
                        ));
                    }
                }
                break;
            }
        }

        Ok(())
    }

    fn create_dest_dir(&mut self) -> io::Result<()> {
        self.dest_dir = self.dest_dir.join(&self.pack_name);
        fs::create_dir_all(&self.dest_dir)?;
        Ok(())
    }

    fn unpack_archive(&self) -> io::Result<()> {
        let bin_file = File::open(&self.file_path)?;
        let decoder = Decoder::new(bin_file)?;
        let mut archive = Archive::new(decoder);

        // Validate paths to prevent directory traversal
        for entry in archive.entries()? {
            let entry = entry?;
            let path = entry.path()?;

            // Check for unsafe paths
            if path.components().any(|c| c.as_os_str() == "..") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsafe path detected: {}", path.display()),
                ));
            }
        }

        // Reset archive and unpack
        let bin_file = File::open(&self.file_path)?;
        let decoder = Decoder::new(bin_file)?;
        let mut archive = Archive::new(decoder);
        archive.unpack(&self.dest_dir)?;

        Ok(())
    }
}

// Helper function to sanitize pack names
fn sanitize_name(name: &str) -> String {
    name.replace(" ", "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "")
        .to_lowercase()
}

// Legacy function for backward compatibility
pub fn install_pack<T: AsRef<Path>>(file: T) -> io::Result<()> {
    let mut installer = PackInstaller::new(file);
    installer.install()
}
