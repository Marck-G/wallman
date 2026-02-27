use crate::trigger::{OutputChange, TriggerResult};
use std::result::Result as StdResult;

/// Apply a batch of wallpaper changes produced by a trigger evaluation.
pub fn apply(result: TriggerResult) -> StdResult<(), Box<dyn std::error::Error>> {
    if result.is_empty() {
        tracing::debug!("apply called with empty TriggerResult — nothing to do");
        return Ok(());
    }

    let mut last_err: Option<Box<dyn std::error::Error>> = None;

    for change in result.changes {
        // Kill existing process for THIS output specifically before starting a new one.
        crate::wallpaper::kill_for_output(&change.output);

        if let Err(e) = apply_to_output(&change) {
            tracing::warn!(
                "Failed to apply wallpaper for output '{}': {}",
                change.output,
                e
            );
            last_err = Some(e);
        }
    }

    if let Some(e) = last_err {
        return Err(e);
    }

    Ok(())
}

/// Apply a wallpaper to a single output using swaybg.
///
/// Spawns `swaybg -o <output> -i <image> -m fill` as a background process.
fn apply_to_output(change: &OutputChange) -> StdResult<(), Box<dyn std::error::Error>> {
    tracing::info!(
        "Applying wallpaper '{}' to output '{}'",
        change.image_path,
        change.output
    );

    // Use spawn() instead of output() so it doesn't block the daemon.
    let child = std::process::Command::new("swaybg")
        .args(&["-o", &change.output, "-i", &change.image_path, "-m", "fill"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    // Register the child so we can kill it later when the wallpaper changes for this output.
    crate::wallpaper::register_process(change.output.clone(), child);

    Ok(())
}
