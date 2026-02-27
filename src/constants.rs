use std::path::PathBuf;

pub fn config_vec() -> Vec<PathBuf> {
    vec![
        dirs::config_dir()
            .unwrap_or(PathBuf::from("/home/"))
            .join("wallman/config"),
        PathBuf::from("/etc/wallman/config"),
        PathBuf::from("/etc/wallman.conf"),
    ]
}

pub fn config_folder() -> PathBuf {
    dirs::config_dir()
        .unwrap_or(PathBuf::from("/home/"))
        .join("wallman/")
}

pub fn data_folder() -> PathBuf {
    dirs::data_local_dir().unwrap().join("wallman/")
}

pub fn decompresion_folder() -> PathBuf {
    data_folder().join("packs/themes")
}

pub fn day_start() -> u32 {
    8
}

pub fn day_end() -> u32 {
    19
}
