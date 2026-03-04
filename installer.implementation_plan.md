Absolutely! Let’s create a **detailed implementation plan in English** for the installer functionality you provided, with a focus on modularity, error handling, and potential refactoring into a struct for cleaner design.

---

# 🔹 Implementation Plan — Pack Installer

## 1. Objective

Develop a robust installer for `.wallman` packs that:

* Reads a `.wallman` file (tar + zstd).
* Extracts `manifest.toml` to get the theme name.
* Creates a target folder in the system (`~/.local/share/wallman/packs/` or similar).
* Extracts all files into that folder.

Optional: refactor into a struct to encapsulate logic and maintain state.

---

## 2. Core Flow

```text
install_pack(file_path)
 ├─ 1. Open .wallman file
 ├─ 2. Decompress zstd
 ├─ 3. Open tar archive
 ├─ 4. Read manifest.toml to determine pack name
 ├─ 5. Sanitize name (remove spaces, special chars)
 ├─ 6. Create destination folder (if not exists)
 ├─ 7. Unpack all files to destination
 └─ 8. Return Result<(), Error>
```

---

## 3. Refactoring Idea: `PackInstaller` Struct

Encapsulate installer logic into a struct to make code cleaner and reusable:

```rust
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
            dest_dir: decompresion_folder(), // base folder, name appended later
        }
    }

    pub fn install(&mut self) -> std::io::Result<()> {
        self.read_manifest()?;
        self.create_dest_dir()?;
        self.unpack_archive()?;
        Ok(())
    }

    fn read_manifest(&mut self) -> std::io::Result<()> { /* ... */ }

    fn create_dest_dir(&mut self) -> std::io::Result<()> { /* ... */ }

    fn unpack_archive(&self) -> std::io::Result<()> { /* ... */ }
}
```

---

## 4. Step-by-Step Details

### Step 1: Open `.wallman` and Decompress

* Use `File::open(file_path)` to open the pack.
* Use `zstd::Decoder` to decompress.
* Wrap decompressed stream into `tar::Archive`.

### Step 2: Read `manifest.toml`

* Iterate over archive entries.
* Look for `manifest.toml`.
* Read contents into string and deserialize with `toml::from_str::<Config>()`.
* Update `pack_name` if `Config.name` exists.
* Sanitize: replace spaces with `-`, remove unsafe characters.

### Step 3: Create Destination Folder

* `dest_dir = base_folder.join(pack_name)`
* Use `fs::create_dir_all(&dest_dir)` to ensure folder exists.

### Step 4: Unpack Archive

* Use `archive.unpack(&dest_dir)?` to extract all files.
* Optional: log skipped files or overwrite warnings.

### Step 5: Error Handling & Robustness

* Handle missing `manifest.toml`: fallback to filename.
* Catch TOML parse errors, provide meaningful messages.
* Validate that files don’t escape the destination folder (`..` paths).
* Optionally, return enum or struct with install summary (files installed, skipped, warnings).

---

## 5. Advantages of Struct Refactor

1. Cleaner separation of concerns:

   * `read_manifest()` → parsing logic.
   * `create_dest_dir()` → filesystem logic.
   * `unpack_archive()` → extraction logic.
2. Easier to extend in future:

   * Add validation hooks (e.g., check images, sandboxing).
   * Track progress for GUI feedback.
3. Encapsulated state: `file_path`, `pack_name`, `dest_dir`.

---

## 6. Optional Enhancements

* **Logging:** Use `tracing` for debug/info logs.
* **Validation:** Check all files are images (or manifest + images) before extracting.
* **GUI Support:** Later the struct can provide callbacks for a progress bar.
* **Flatpak Path Handling:** Ensure extraction works in sandboxed environment.

---

## 7. Roadmap

1. Refactor `install_pack()` into `PackInstaller`.
2. Implement `read_manifest()` with proper error handling and sanitization.
3. Implement `create_dest_dir()`.
4. Implement `unpack_archive()`.
5. Write unit tests:

   * Normal pack.
   * Pack with missing manifest.
   * Pack with invalid TOML.
   * Pack containing unsafe paths.
6. Integrate `tracing` for logging.
7. Ensure Flatpak compatibility.

---

If you want, I can **write the full Rust implementation of `PackInstaller`**, complete with manifest parsing, safe extraction, and error handling, ready to drop into Wallman.

Do you want me to do that next?
