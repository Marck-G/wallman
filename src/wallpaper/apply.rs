use crate::config::FillMode;
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
        // Tear down existing surface for this output before creating a new one.
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

/// Spawn a background thread that creates a `zwlr_layer_surface_v1` on the
/// BACKGROUND layer for the given output and keeps it alive.
fn apply_to_output(change: &OutputChange) -> StdResult<(), Box<dyn std::error::Error>> {
    tracing::info!(
        "Applying wallpaper '{}' to output '{}' (mode: {:?})",
        change.image_path,
        change.output,
        change.fill_mode,
    );

    // Load and scale the image on the calling thread so we surface errors early.
    let rgba = load_image(&change.image_path, &change.fill_mode, None, None)?;
    let fill_mode = change.fill_mode.clone();

    let output_name = change.output.clone();
    let (stop_tx, stop_rx) = std::sync::mpsc::sync_channel::<()>(1);

    let thread = std::thread::Builder::new()
        .name(format!("wallpaper-{}", output_name))
        .spawn(move || {
            if let Err(e) = run_surface(output_name.clone(), rgba, fill_mode, stop_rx) {
                tracing::error!(
                    "Wayland surface thread for '{}' exited with error: {}",
                    output_name,
                    e
                );
            }
        })?;

    let handle = crate::wallpaper::WallpaperHandle {
        stop_tx,
        thread: Some(thread),
    };

    crate::wallpaper::register_handle(change.output.clone(), handle);
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Image loading & scaling
// ─────────────────────────────────────────────────────────────────────────────

/// Load an image file and return raw BGRA pixels at the requested output
/// dimensions (if known). If dimensions are not yet known they are deduced
/// after the compositor sends the first `configure` event; pass `None` here
/// and call `scale_image` later.
pub fn load_image(
    path: &str,
    fill_mode: &FillMode,
    width: Option<u32>,
    height: Option<u32>,
) -> StdResult<RgbaImage, Box<dyn std::error::Error>> {
    let img = image::open(path)?.into_rgba8();

    let result = if let (Some(w), Some(h)) = (width, height) {
        scale_image(img, fill_mode, w, h)
    } else {
        // Return original — will be re-scaled once we know the output size.
        img
    };

    let (width, height) = result.dimensions();
    Ok(RgbaImage { pixels: result.into_raw(), width, height })
}

/// Scale a decoded RGBA image according to the requested fill mode.
pub fn scale_image(
    src: image::RgbaImage,
    fill_mode: &FillMode,
    target_w: u32,
    target_h: u32,
) -> image::RgbaImage {
    use image::imageops::{self, FilterType};
    use image::RgbaImage as Img;

    let (src_w, src_h) = src.dimensions();

    match fill_mode {
        // Cover: scale so the image fills the output, then center-crop.
        FillMode::Fill | FillMode::Crop => {
            let scale = f64::max(
                target_w as f64 / src_w as f64,
                target_h as f64 / src_h as f64,
            );
            let new_w = (src_w as f64 * scale).ceil() as u32;
            let new_h = (src_h as f64 * scale).ceil() as u32;
            let resized = imageops::resize(&src, new_w, new_h, FilterType::Lanczos3);
            let x = (new_w.saturating_sub(target_w)) / 2;
            let y = (new_h.saturating_sub(target_h)) / 2;
            imageops::crop_imm(&resized, x, y, target_w, target_h).to_image()
        }

        // Contain: scale so the image fits inside the output, letterbox with black.
        FillMode::Fit => {
            let scale = f64::min(
                target_w as f64 / src_w as f64,
                target_h as f64 / src_h as f64,
            );
            let new_w = (src_w as f64 * scale).round() as u32;
            let new_h = (src_h as f64 * scale).round() as u32;
            let resized = imageops::resize(&src, new_w, new_h, FilterType::Lanczos3);
            let mut canvas = Img::new(target_w, target_h); // black by default
            let x = (target_w.saturating_sub(new_w)) / 2;
            let y = (target_h.saturating_sub(new_h)) / 2;
            imageops::overlay(&mut canvas, &resized, x as i64, y as i64);
            canvas
        }

        // Stretch: ignore aspect ratio.
        FillMode::Scale => {
            imageops::resize(&src, target_w, target_h, FilterType::Lanczos3)
        }

        // Tile: repeat the original at its natural size.
        FillMode::Tile => {
            let mut canvas = Img::new(target_w, target_h);
            let mut y = 0i64;
            while y < target_h as i64 {
                let mut x = 0i64;
                while x < target_w as i64 {
                    imageops::overlay(&mut canvas, &src, x, y);
                    x += src_w as i64;
                }
                y += src_h as i64;
            }
            canvas
        }
    }
}

/// Intermediate decoded image buffer.
pub struct RgbaImage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Wayland surface event loop
// ─────────────────────────────────────────────────────────────────────────────

use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_buffer, wl_compositor, wl_output, wl_registry, wl_shm, wl_shm_pool, wl_surface,
    },
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1},
};

struct WallpaperState {
    compositor: Option<wl_compositor::WlCompositor>,
    shm: Option<wl_shm::WlShm>,
    layer_shell: Option<ZwlrLayerShellV1>,

    /// The `wl_output` matching the requested output name.
    target_output: Option<wl_output::WlOutput>,
    target_name: String,

    surface: Option<wl_surface::WlSurface>,
    layer_surface: Option<ZwlrLayerSurfaceV1>,

    /// Raw RGBA pixel data (will be converted to BGRA for wl_shm ARGB8888).
    rgba: RgbaImage,
    /// Fill mode — used when the compositor sends us the actual dimensions.
    fill_mode: FillMode,

    configured: bool,
    running: bool,
}

impl WallpaperState {
    fn new(target_name: String, rgba: RgbaImage, fill_mode: FillMode) -> Self {
        Self {
            compositor: None,
            shm: None,
            layer_shell: None,
            target_output: None,
            target_name,
            surface: None,
            layer_surface: None,
            rgba,
            fill_mode,
            configured: false,
            running: true,
        }
    }

    /// Once all globals are bound and we know the target output, create the
    /// layer surface.
    fn try_create_surface(&mut self, qh: &QueueHandle<Self>) {
        if self.surface.is_some() {
            return;
        }
        let (Some(compositor), Some(layer_shell), Some(output)) = (
            self.compositor.as_ref(),
            self.layer_shell.as_ref(),
            self.target_output.as_ref(),
        ) else {
            return;
        };

        let surface = compositor.create_surface(qh, ());
        let layer_surface = layer_shell.get_layer_surface(
            &surface,
            Some(output),
            zwlr_layer_shell_v1::Layer::Background,
            "wallman".to_string(),
            qh,
            (),
        );

        // Request to fill the full output.
        layer_surface.set_size(0, 0); // 0 = fill
        layer_surface.set_exclusive_zone(-1); // don't affect other surfaces
        layer_surface.set_anchor(
            zwlr_layer_surface_v1::Anchor::Top
                | zwlr_layer_surface_v1::Anchor::Bottom
                | zwlr_layer_surface_v1::Anchor::Left
                | zwlr_layer_surface_v1::Anchor::Right,
        );
        surface.commit();

        self.surface = Some(surface);
        self.layer_surface = Some(layer_surface);
    }

    /// Render the image into a wl_shm buffer and attach it to the surface.
    fn render(&mut self, width: u32, height: u32, qh: &QueueHandle<Self>) {
        let shm = match self.shm.as_ref() {
            Some(s) => s,
            None => return,
        };
        let surface = match self.surface.as_ref() {
            Some(s) => s,
            None => return,
        };

        // Scale/compose the image to the actual output dimensions.
        let src = image::RgbaImage::from_raw(
            self.rgba.width,
            self.rgba.height,
            self.rgba.pixels.clone(),
        )
        .expect("invalid pixel buffer");

        let scaled = scale_image(src, &self.fill_mode, width, height);

        // Convert RGBA → BGRA (wl_shm ARGB8888 is stored as BGRA in little-endian).
        let mut bgra: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
        for pixel in scaled.pixels() {
            bgra.push(pixel[2]); // B
            bgra.push(pixel[1]); // G
            bgra.push(pixel[0]); // R
            bgra.push(pixel[3]); // A
        }

        let stride = width * 4;
        let size = (stride * height) as usize;

        // Create anonymous shared memory file.
        let shm_fd = create_shm_fd(size).expect("failed to create shm fd");
        // Write pixels.
        {
            use std::io::Write;
            let mut file = unsafe { <std::fs::File as std::os::unix::io::FromRawFd>::from_raw_fd(shm_fd) };
            file.write_all(&bgra).expect("failed to write pixels");
            // Don't close — wl_shm_pool keeps its own ref via mmap.
            std::mem::forget(file);
        }

        let pool = shm.create_pool(
            unsafe { std::os::unix::io::BorrowedFd::borrow_raw(shm_fd) },
            size as i32,
            qh,
            (),
        );
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            wl_shm::Format::Argb8888,
            qh,
            (),
        );
        pool.destroy();
        unsafe { libc::close(shm_fd) };

        surface.attach(Some(&buffer), 0, 0);
        surface.damage_buffer(0, 0, width as i32, height as i32);
        surface.commit();
    }
}

// ── Dispatch impls ────────────────────────────────────────────────────────────

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WallpaperState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &GlobalListContents,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            match interface.as_str() {
                "wl_compositor" => {
                    state.compositor = Some(
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, version.min(4), qh, ()),
                    );
                }
                "wl_shm" => {
                    state.shm = Some(
                        registry.bind::<wl_shm::WlShm, _, _>(name, version.min(1), qh, ()),
                    );
                }
                "zwlr_layer_shell_v1" => {
                    state.layer_shell = Some(
                        registry.bind::<ZwlrLayerShellV1, _, _>(name, version.min(4), qh, ()),
                    );
                }
                "wl_output" => {
                    // Bind the output; we'll match the name in its events.
                    let output = registry.bind::<wl_output::WlOutput, _, _>(name, version.min(4), qh, ());
                    // Tentatively store as candidate; the event handler resolves name.
                    if state.target_output.is_none() {
                        state.target_output = Some(output);
                    }
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for WallpaperState {
    fn event(
        state: &mut Self,
        output: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        // wl_output v4 sends Name events; earlier versions only send Description.
        match event {
            wl_output::Event::Name { name } => {
                if name == state.target_name {
                    state.target_output = Some(output.clone());
                    state.try_create_surface(qh);
                } else if state.target_output.as_ref().map(|o| o == output).unwrap_or(false) {
                    state.target_output = None;
                }
            }
            wl_output::Event::Done => {
                state.try_create_surface(qh);
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for WallpaperState {
    fn event(_: &mut Self, _: &wl_compositor::WlCompositor, _: wl_compositor::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<wl_surface::WlSurface, ()> for WallpaperState {
    fn event(_: &mut Self, _: &wl_surface::WlSurface, _: wl_surface::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<wl_shm::WlShm, ()> for WallpaperState {
    fn event(_: &mut Self, _: &wl_shm::WlShm, _: wl_shm::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for WallpaperState {
    fn event(_: &mut Self, _: &wl_shm_pool::WlShmPool, _: wl_shm_pool::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<wl_buffer::WlBuffer, ()> for WallpaperState {
    fn event(_: &mut Self, buffer: &wl_buffer::WlBuffer, event: wl_buffer::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        if let wl_buffer::Event::Release = event {
            buffer.destroy();
        }
    }
}

impl Dispatch<ZwlrLayerShellV1, ()> for WallpaperState {
    fn event(_: &mut Self, _: &ZwlrLayerShellV1, _: zwlr_layer_shell_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for WallpaperState {
    fn event(
        state: &mut Self,
        layer_surface: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure { serial, width, height } => {
                layer_surface.ack_configure(serial);
                state.configured = true;
                state.render(width, height, qh);
            }
            zwlr_layer_surface_v1::Event::Closed => {
                tracing::info!("Layer surface closed by compositor");
                state.running = false;
            }
            _ => {}
        }
    }
}

// ── Surface thread entry point ────────────────────────────────────────────────

fn run_surface(
    output_name: String,
    rgba: RgbaImage,
    fill_mode: FillMode,
    stop_rx: std::sync::mpsc::Receiver<()>,
) -> StdResult<(), Box<dyn std::error::Error>> {
    let conn = Connection::connect_to_env()?;
    let (globals, mut queue) = registry_queue_init::<WallpaperState>(&conn)?;
    let qh = queue.handle();

    let mut state = WallpaperState::new(output_name, rgba, fill_mode);

    // Bind globals that are already in the registry.
    if let Ok(compositor) = globals.bind::<wl_compositor::WlCompositor, _, _>(&qh, 1..=4, ()) {
        state.compositor = Some(compositor);
    }
    if let Ok(shm) = globals.bind::<wl_shm::WlShm, _, _>(&qh, 1..=1, ()) {
        state.shm = Some(shm);
    }
    if let Ok(layer_shell) = globals.bind::<ZwlrLayerShellV1, _, _>(&qh, 1..=4, ()) {
        state.layer_shell = Some(layer_shell);
    }

    // Enumerate outputs from the registry.
    let outputs: Vec<wl_output::WlOutput> = globals
        .contents()
        .clone_list()
        .iter()
        .filter(|g| g.interface == "wl_output")
        .map(|g| globals.registry().bind::<wl_output::WlOutput, _, _>(g.name, g.version.min(4), &qh, ()))
        .collect();

    // Initial roundtrip to receive output names and other globals.
    queue.roundtrip(&mut state)?;
    queue.roundtrip(&mut state)?;

    // If no output matched by name, fall back to first available.
    if state.target_output.is_none() && !outputs.is_empty() {
        tracing::warn!(
            "Output '{}' not found by name — using first available output",
            state.target_name
        );
        state.target_output = Some(outputs[0].clone());
        state.try_create_surface(&qh);
        queue.roundtrip(&mut state)?;
    }

    // Event loop — runs until stop signal or compositor closes the surface.
    use std::os::fd::AsFd;
    let display_fd = conn.as_fd();
    loop {
        // Check stop signal (non-blocking).
        if stop_rx.try_recv().is_ok() {
            tracing::debug!("Wallpaper surface for '{}' stopping on request", state.target_name);
            break;
        }
        if !state.running {
            break;
        }

        queue.flush()?;

        // Wait for events with a short timeout so we can poll the stop channel.
        let mut poll_fds = [nix::poll::PollFd::new(
            display_fd,
            nix::poll::PollFlags::POLLIN,
        )];
        let _ = nix::poll::poll(&mut poll_fds, 200u16);

        queue.dispatch_pending(&mut state)?;
    }

    // Clean up layer surface.
    if let Some(ls) = state.layer_surface.take() {
        ls.destroy();
    }
    if let Some(s) = state.surface.take() {
        s.destroy();
    }
    queue.flush()?;

    Ok(())
}

// ── Shared memory helper ──────────────────────────────────────────────────────

/// Create an anonymous memfd (Linux) or shm_open file (other POSIX) of the
/// requested size. Returns a raw file descriptor.
fn create_shm_fd(size: usize) -> StdResult<std::os::unix::io::RawFd, Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        let fd = unsafe {
            libc::syscall(
                libc::SYS_memfd_create,
                b"wallman-wl\0".as_ptr(),
                libc::MFD_CLOEXEC,
            ) as libc::c_int
        };
        if fd < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let ret = unsafe { libc::ftruncate(fd, size as libc::off_t) };
        if ret < 0 {
            unsafe { libc::close(fd) };
            return Err(std::io::Error::last_os_error().into());
        }
        return Ok(fd);
    }

    #[cfg(not(target_os = "linux"))]
    {
        use std::ffi::CString;
        let name = CString::new(format!("/wallman-wl-{}", std::process::id()))?;
        let fd = unsafe {
            libc::shm_open(
                name.as_ptr(),
                libc::O_CREAT | libc::O_RDWR | libc::O_CLOEXEC,
                0o600,
            )
        };
        if fd < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        unsafe { libc::shm_unlink(name.as_ptr()) };
        let ret = unsafe { libc::ftruncate(fd, size as libc::off_t) };
        if ret < 0 {
            unsafe { libc::close(fd) };
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(fd)
    }
}
