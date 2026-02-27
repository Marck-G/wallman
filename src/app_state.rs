use crate::Config;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

pub struct AppState {
    pub config_path: String,
    pub images_pool: Option<String>,
    pub is_pool: bool,
    pub config: Config,
}

// Global application state using OnceLock for lazy initialization
pub static APP_STATE: OnceLock<Arc<Mutex<AppState>>> = OnceLock::new();

impl Default for AppState {
    fn default() -> Self {
        Self {
            config_path: crate::constants::config_folder()
                .to_string_lossy()
                .to_string(),
            images_pool: None,
            is_pool: false,
            config: Config::default(),
        }
    }
}

impl AppState {
    pub fn new(
        config: Config,
        config_path: String,
        images_pool: Option<String>,
        is_pool: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(AppState {
            config_path,
            images_pool,
            is_pool,
            config,
        })
    }

    // pub fn new_with_config(config: Config, ) -> Self {
    //     let config_path = crate::constants::config_folder().to_string_lossy().to_string();
    //     let images_pool = config.poll.clone();
    //     let is_pool = config.poll.is_some();

    //     AppState {
    //         config_path,
    //         images_pool,
    //         is_pool,
    //         config,
    //     }
    // }

    pub fn get_instance() -> Arc<Mutex<AppState>> {
        APP_STATE.get().unwrap().clone()
    }

    pub fn get_current_background(&self) -> Option<&str> {
        // For now, return the first background image if available
        self.config
            .background
            .as_ref()
            .and_then(|bg| bg.values().next())
            .and_then(|config| config.image.as_deref())
    }

    pub fn get_fill_mode(&self) -> crate::config::FillMode {
        self.config
            .background
            .as_ref()
            .and_then(|bg| bg.values().next())
            .map(|config| config.fill_mode.clone())
            .unwrap_or(crate::config::FillMode::Fill)
    }

    pub fn update_background(&mut self, image_path: String, fill_mode: crate::config::FillMode) {
        let background_config = crate::config::BackgroundConfig {
            image: Some(image_path),
            fill_mode,
        };

        self.config.background = Some(std::collections::HashMap::from([(
            "default".to_string(),
            background_config,
        )]));
    }

    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = crate::constants::config_folder().join("config.toml");
        self.config.save_to_file(&config_path)
    }

    pub fn reload_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = Config::load(PathBuf::from(&self.config_path))?;

        // If a theme pool is active, merge its manifest settings.
        if let Some(pool) = &config.pool {
            let pool_path = PathBuf::from(pool);
            if let Err(e) = config.merge_theme(pool_path) {
                tracing::warn!("Failed to merge theme manifest during reload: {}", e);
            }
        }

        self.config = config;
        self.images_pool = self.config.pool.clone();
        self.is_pool = self.config.pool.is_some();
        Ok(())
    }

    /// Resolve an image path against the current theme pool if it is relative.
    pub fn resolve_image_path(&self, path: &str) -> String {
        let p = std::path::Path::new(path);
        if p.is_absolute() {
            return path.to_string();
        }

        if let Some(pool) = &self.images_pool {
            let pool_path = std::path::Path::new(pool);
            // Themes usually have an 'images' subfolder.
            let theme_images = pool_path.join("images");
            let final_path = if theme_images.exists() {
                theme_images.join(path)
            } else {
                pool_path.join(path)
            };
            return final_path.to_string_lossy().to_string();
        }

        path.to_string()
    }
}
