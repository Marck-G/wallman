use std::{
    process::{Command, Stdio},
};

pub fn change_wallpaper<T: AsRef<str>>(wallpaper: T, fill_mode: T) {
    // Mata swaybg anterior (si existe)
    let _ = Command::new("pkill")
        .arg("swaybg")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    Command::new("swaybg")
        .args([
            "-i",
            wallpaper.as_ref(),
            "-m",
            fill_mode.as_ref(), // stretch, center, tile, etc.
        ])
        .spawn()
        .unwrap();
}
