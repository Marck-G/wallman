# Wallman — CLI Implementation Plan

---

## 1. Objective

Design and implement a **command-line interface (CLI)** for Wallman that allows users to:

* Manage themes (`.wallman` packs)
* Create and package themes
* Install and list themes
* Configure wallpapers and triggers
* Control the daemon lifecycle
* Provide a stable interface that will later be reused by the GUI (v2.0)

The CLI must act as the **primary user interface for the MVP** and should be designed as a thin orchestration layer over internal modules.

---

## 2. Design Principles

The CLI must follow these rules:

1. **CLI is not business logic**

   * It only calls application services/modules.
   * All real logic lives in library modules.

2. **Deterministic commands**

   * Same input → same result.
   * No hidden state mutations.

3. **Daemon-first architecture**

   * Commands either:

     * operate on filesystem (themes/config)
     * communicate with daemon (future IPC).

4. **Extensible command tree**

   * Adding commands must not require refactoring existing ones.

---

## 3. CLI Architecture

### High-Level Flow

```
CLI Parser
    ↓
Command Dispatcher
    ↓
Application Services
    ↓
Core Modules (packager, installer, triggers, config, daemon)
```

---

## 4. Folder Structure

Create a dedicated CLI module:

```
src/
 └── cli/
      ├── mod.rs
      ├── app.rs              // CLI definition
      ├── commands/
      │     ├── mod.rs
      │     ├── theme.rs
      │     ├── daemon.rs
      │     ├── config.rs
      │     └── pack.rs
      └── dispatcher.rs
```

---

## 5. Command Parser

The CLI must define a hierarchical command structure.

### Root command

```
wallman <COMMAND> [OPTIONS]
```

---

## 6. Command Groups

---

### 6.1 Theme Commands

#### Purpose

Manage installed themes and creation workflows.

#### Commands

```
wallman theme create <path>
wallman theme pack <path>
wallman theme install <file.wallman>
wallman theme list
wallman theme set <theme-name>
wallman theme remove <theme-name>
```

---

#### Responsibilities

**create**

* Generate template directory:

  * manifest.toml
  * images/
* Populate minimal config.

**pack**

* Calls `Packager::pack()`.

**install**

* Calls installer module.
* Validates pack before extraction.

**list**

* Reads installed themes directory.
* Displays metadata.

**set**

* Updates user config.
* Triggers daemon reload (future).

---

---

### 6.2 Daemon Commands

#### Purpose

Control runtime behavior.

```
wallman daemon start
wallman daemon stop
wallman daemon restart
wallman daemon status
```

---

#### Responsibilities

* Start background process.
* Prevent duplicate daemon instances.
* Report running state.

Future:

* IPC communication.

---

---

### 6.3 Config Commands

#### Purpose

Manage user configuration.

```
wallman config init
wallman config edit
wallman config validate
wallman config path
```

---

#### Responsibilities

**init**

* Create default config file.

**edit**

* Open editor using `$EDITOR`.

**validate**

* Parse TOML.
* Validate triggers and outputs.

**path**

* Print config location.

---

---

### 6.4 Pack Commands (Optional Separation)

If desired:

```
wallman pack build
wallman pack inspect <file.wallman>
```

Used mainly for debugging and development.

---

## 7. Dispatcher Layer

The dispatcher connects parsed CLI commands to application services.

Example responsibility:

```
match command {
   Theme(Create) => theme::create(),
   Theme(Install) => installer::install(),
   Daemon(Start) => daemon::start(),
}
```

Rules:

* No filesystem logic here.
* Only orchestration.

---

## 8. Output & UX Rules

### Success Output

* Short and clear.
* Example:

```
Theme installed successfully: nord-dark
```

### Errors

* Human readable.
* Avoid Rust debug dumps.

Example:

```
Error: manifest.toml not found in pack
```

---

## 9. Logging Integration

CLI must support verbosity flags:

```
wallman --verbose
wallman --debug
```

Behavior:

* default → minimal output
* verbose → operational logs
* debug → tracing enabled

---

## 10. Exit Codes

Standardized exit codes:

| Code | Meaning        |
| ---- | -------------- |
| 0    | Success        |
| 1    | Generic error  |
| 2    | Invalid config |
| 3    | Pack error     |
| 4    | Daemon error   |

---

## 11. Configuration Interaction

CLI must never mutate config manually.

Instead:

1. Load config module.
2. Modify struct.
3. Serialize back to TOML.
4. Write atomically.

Atomic write required:

* write temp file
* rename.

---

## 12. Integration With Multi-Monitor System

Commands affecting wallpapers must:

* NOT compute outputs.
* Delegate to daemon/triggers.

CLI only modifies configuration.

---

## 13. Future GUI Compatibility Requirement

Every CLI command must correspond to a callable internal function.

Example:

```
theme::install(path)
```

Later reused by:

```
GUI → API → same function
```

No duplicated logic allowed.

---

## 14. Error Handling Strategy

Use structured errors internally.

CLI layer converts them into:

* readable messages
* exit codes.

---

## 15. Implementation Order (Strict)

### Phase 1 — Skeleton

* Create CLI module
* Define root command
* Implement dispatcher

### Phase 2 — Theme Workflow

* theme create
* theme pack
* theme install
* theme list

### Phase 3 — Config Commands

* init
* validate
* edit

### Phase 4 — Daemon Commands

* start
* stop
* status

### Phase 5 — UX Improvements

* verbosity flags
* formatted output
* error normalization

---

## 16. Validation Requirements

Before MVP release verify:

* Theme lifecycle works end-to-end
* Invalid packs fail safely
* Config corruption is impossible
* CLI works inside Flatpak sandbox

---

## 17. Expected Result

After completion:

* Wallman becomes fully usable without GUI.
* CLI acts as stable automation interface.
* Future GUI becomes a thin frontend.
* Architecture remains clean and modular.

---
