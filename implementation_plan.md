# Wallman — System Integration Implementation Plan

*(Unifying Core, Packs, Triggers, Multi-Monitor and CLI)*

---

## 1. Objective

Integrate all previously designed components into a **cohesive runnable application**:

* Configuration system
* Global application state (`AppState`)
* Theme packager & installer
* Trigger engine
* Multi-monitor output resolver
* Wallpaper backend (swaybg for MVP)
* CLI interface
* Daemon runtime

The result must be a **fully working MVP daemon controlled through the CLI**, with clean boundaries that allow future migration to Wayland layers and GUI support.

---

## 2. Target Runtime Architecture

```
CLI
 │
 ▼
Command Dispatcher
 │
 ├── Filesystem Operations (themes/config)
 └── Daemon Control
        │
        ▼
   Wallman Daemon
        │
        ├── AppState (global shared state)
        ├── TriggerManager
        │       ├── StaticTrigger
        │       ├── DayTimeTrigger
        │       └── WeatherTrigger
        │
        ├── OutputResolver
        └── Wallpaper Backend (swaybg)
```

---

## 3. Integration Principles

1. **Single source of truth**

   * All runtime data comes from `AppState`.

2. **CLI never performs runtime logic**

   * CLI modifies configuration or controls daemon only.

3. **Triggers never call system commands**

   * Only produce decisions.

4. **Wallpaper backend isolated**

   * Allows compositor replacement later.

5. **Daemon owns execution loop**

---

## 4. Final Project Structure

```
src/
 ├── main.rs
 ├── state/
 │     └── app_state.rs
 ├── config/
 ├── cli/
 ├── triggers/
 ├── outputs/
 ├── wallpaper/
 ├── packager/
 ├── installer/
 └── daemon/
       └── runtime.rs
```

---

## 5. Step 1 — Initialize Application State

### Responsibilities

* Load config file.
* Resolve config path.
* Initialize global `APP_STATE`.

### Required Work

1. Implement config loader:

   * Read TOML.
   * Deserialize into `Config`.
2. Build `AppState::new`.
3. Store inside:

```rust
APP_STATE.set(Arc::new(Mutex::new(state)));
```

### Validation

* Application must fail early if config is invalid.

---

## 6. Step 2 — Daemon Runtime Module

Create:

```
daemon/runtime.rs
```

### Responsibilities

* Bootstrap system.
* Initialize triggers.
* Start trigger manager loop.

---

### Runtime Flow

```
load config
initialize AppState
detect outputs
build triggers
start TriggerManager
block main thread
```

---

## 7. Step 3 — Trigger Registration

Inside daemon startup:

1. Inspect config.
2. Register only required triggers.

Example logic:

```
if background exists → StaticTrigger
if timeConfig exists → DayTimeTrigger
if weather exists → WeatherTrigger
```

---

### Required Implementation

Create builder:

```
fn build_triggers(config: &Config) -> Vec<Box<dyn Trigger>>
```

This prevents logic duplication in `main`.

---

## 8. Step 4 — Integrate Multi-Monitor Resolver

Trigger evaluation must now follow:

```
clone config
detect outputs
resolve wildcard mappings
compute per-output decisions
return batch result
```

### Required Work

* Implement `OutputResolver`.
* Integrate resolver calls inside triggers.
* Ensure wildcard `*` fallback works.

---

## 9. Step 5 — Wallpaper Backend Integration

Create unified entry:

```
wallpaper::apply(TriggerResult)
```

Responsibilities:

1. Iterate output changes.
2. Spawn swaybg per output.
3. Handle failures safely.

MVP behavior:

* Replace wallpaper immediately.
* Ignore disconnected outputs.

---

## 10. Step 6 — Connect CLI to Runtime

### CLI must support:

```
wallman daemon start
```

Implementation:

1. CLI launches daemon runtime.
2. Runtime blocks execution.
3. Logs printed via tracing.

Future:

* Replace with IPC instead of direct execution.

---

## 11. Step 7 — Theme Lifecycle Integration

### Packager Integration

CLI → `theme pack`

* Calls Packager module.

### Installer Integration

CLI → `theme install`

* Calls installer.
* Extracts into themes directory.

### Runtime Usage

Triggers must resolve image paths relative to installed theme folder.

---

## 12. Step 8 — Config Reload Foundation

Even before hot reload:

* Runtime must reload config on startup.
* All triggers must read config dynamically from `AppState`.

Requirement:
Triggers must NOT cache config permanently.

---

## 13. Step 9 — Logging Integration

Enable global tracing initialization in daemon startup.

Log events:

* daemon start
* trigger evaluation
* wallpaper change
* API failures
* pack installation

---

## 14. Step 10 — Error Propagation

Rules:

* Core modules return structured errors.
* Daemon logs errors but continues when possible.
* CLI converts errors into user messages.

---

## 15. Step 11 — Execution Modes

Main must support:

### CLI Mode

```
wallman theme install ...
```

### Daemon Mode

```
wallman daemon start
```

Implementation:

```
parse CLI
if daemon command → run runtime
else → execute command
```

---

## 16. Step 12 — MVP Validation Checklist

System must successfully:

* Load config
* Install theme
* Detect monitors
* Resolve wildcard outputs
* Run triggers
* Apply wallpapers per monitor
* Run continuously without CPU spikes

---

## 17. Integration Order (STRICT)

### Phase 1 — State & Config

* AppState initialization
* Config loading

### Phase 2 — Runtime

* Daemon module
* TriggerManager integration

### Phase 3 — Outputs

* Output detection
* Wildcard resolver

### Phase 4 — Wallpaper

* Apply pipeline

### Phase 5 — CLI

* daemon start
* theme install/set

### Phase 6 — Full End-to-End Test

---

## 18. Expected Final Result

After integration:

✅ Wallman runs as a real daemon
✅ CLI fully controls workflow
✅ Multi-monitor works automatically
✅ Triggers operate independently
✅ Themes install and activate correctly
✅ Architecture ready for Wayland layer backend and GUI v2.0

---
