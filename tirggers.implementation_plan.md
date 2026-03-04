Perfect — here is the **expanded and explicit English implementation plan**, rewritten as a **clear execution guide** for building the trigger system in Rust.
This version focuses on **what must be implemented step-by-step**, how it integrates with `main`, and how new triggers can be added safely.

---

# ✅ Wallman — Trigger System Implementation Plan

## 1. Goal

Implement a **modular trigger system** written in Rust that:

* Reads configuration from the global `AppState`
* Evaluates conditions periodically
* Decides which wallpaper should be applied
* Can be easily extended with new triggers
* Can be registered directly from `main.rs`

Initial triggers:

1. Static wallpaper trigger
2. Day/Night trigger
3. Weather trigger

---

## 2. High-Level Design

We will implement a **trait-based trigger architecture**.

Each trigger:

* Receives access to `AppState`
* Runs periodically
* Returns a decision (if wallpaper must change)

Triggers DO NOT apply wallpapers themselves.

Instead:

```
Trigger → produces decision → Wallpaper module applies change
```

This prevents tight coupling and allows future Wayland migration.

---

## 3. Folder Structure (must be created)

Create a new module:

```
src/
 └── triggers/
      ├── mod.rs
      ├── trigger.rs
      ├── manager.rs
      ├── static_trigger.rs
      ├── daytime_trigger.rs
      └── weather_trigger.rs
```

---

## 4. Step 1 — Define the Trigger Trait

Create:

`triggers/trigger.rs`

This trait defines the contract every trigger must implement.

```rust
pub trait Trigger: Send {
    /// Called once when trigger starts
    fn init(&mut self) -> anyhow::Result<()>;

    /// Called periodically by the manager
    fn evaluate(&mut self) -> anyhow::Result<Option<TriggerResult>>;

    /// Evaluation interval in seconds
    fn interval(&self) -> u64;
}
```

---

### Trigger Result Object

Also define:

```rust
pub struct TriggerResult {
    pub output: String,
    pub image_path: String,
}
```

This represents a wallpaper change request.

---

## 5. Step 2 — Accessing Global AppState Safely

Triggers must read configuration from:

```rust
static APP_STATE: OnceLock<Arc<Mutex<AppState>>>;
```

IMPORTANT RULE:

✅ Lock only briefly
❌ Never perform IO or network operations while holding the mutex.

Correct pattern:

```rust
let state = APP_STATE.get().unwrap().lock().unwrap();
let config = state.config.clone();
drop(state);
```

---

## 6. Step 3 — Implement Trigger Manager

Create:

`triggers/manager.rs`

### Responsibilities

The manager:

* Stores all triggers
* Runs evaluation loops
* Executes triggers at the correct interval
* Applies wallpaper changes

---

### Structure

```rust
pub struct TriggerManager {
    triggers: Vec<ScheduledTrigger>,
}
```

---

### Scheduled Wrapper

```rust
use std::time::Instant;

pub struct ScheduledTrigger {
    pub trigger: Box<dyn Trigger>,
    pub next_run: Instant,
}
```

---

### Manager Loop

```rust
pub fn run(&mut self) -> anyhow::Result<()> {
    loop {
        let now = Instant::now();

        for scheduled in self.triggers.iter_mut() {
            if now >= scheduled.next_run {
                if let Some(result) = scheduled.trigger.evaluate()? {
                    wallpaper::apply(result)?;
                }

                scheduled.next_run =
                    now + Duration::from_secs(scheduled.trigger.interval());
            }
        }

        std::thread::sleep(Duration::from_millis(500));
    }
}
```

---

## 7. Step 4 — Static Trigger Implementation

Create:

`static_trigger.rs`

### What it must do

* Read `[background]` config
* Emit wallpaper once
* Never trigger again

---

### Internal State

```rust
pub struct StaticTrigger {
    executed: bool,
}
```

---

### Behavior

1. On first evaluation → return wallpaper.
2. Set `executed = true`.
3. Future evaluations return `None`.

---

## 8. Step 5 — DayTime Trigger Implementation

Create:

`daytime_trigger.rs`

### Required Logic

1. Read `timeConfig` from config, day and night is the hours that start each.
2. Determine current local time.
3. Decide whether it is DAY or NIGHT.
4. Detect state change.
5. Emit result only when state changes.

---

### Internal State

```rust
pub struct DayTimeTrigger {
    last_state: Option<bool>, // true = day
}
```

---

### Evaluation Interval

```
interval() = 60 seconds
```

Checking every minute is sufficient.

---

## 9. Step 6 — Weather Trigger Implementation

Create:

`weather_trigger.rs`

### Required Steps

1. Read latitude and longitude.
2. Call Open-Meteo API.
3. Convert response → `WeatherStates`.
4. Select configured image.
5. Emit change only if weather changed.

---

### Internal State

```rust
pub struct WeatherTrigger {
    last_weather: Option<WeatherStates>,
}
```

---

### Evaluation Interval

```
interval() = 600–1800 seconds
```

Avoid API rate limits.

---

## 10. Step 7 — Wallpaper Application Module

Create separate module:

```
src/wallpaper/apply.rs
```

```rust
pub fn apply(result: TriggerResult) -> anyhow::Result<()> {
    // call swaybg for now
}
```

IMPORTANT:

Triggers must NEVER call swaybg directly.

This isolates compositor logic.

---

## 11. Step 8 — Register Triggers in `main.rs`

Triggers can be added directly in `main`.

Example:

```rust
use triggers::{
    manager::TriggerManager,
    static_trigger::StaticTrigger,
    daytime_trigger::DayTimeTrigger,
    weather_trigger::WeatherTrigger,
};

fn main() -> anyhow::Result<()> {
    let mut manager = TriggerManager::new();

    manager.add(Box::new(StaticTrigger::new()));
    manager.add(Box::new(DayTimeTrigger::new()));
    manager.add(Box::new(WeatherTrigger::new()));

    manager.run()?;

    Ok(())
}
```

Adding a new trigger later becomes:

```
1 file + 1 line in main
```

---

## 12. Step 9 — Logging (Required)

Use `tracing` inside triggers:

* trigger start
* evaluation result
* skipped execution
* API failures

Example:

```rust
tracing::info!("Weather changed → applying new wallpaper");
```

---

## 13. Step 10 — Implementation Order (STRICT)

Follow this order:

### Phase 1

* Create folder structure
* Implement Trigger trait
* Implement TriggerManager
* Implement StaticTrigger

### Phase 2

* Implement DayTimeTrigger
* Add scheduler logic

### Phase 3

* Implement WeatherTrigger
* Add HTTP client + timeout handling

### Phase 4

* Connect wallpaper module
* Add tracing logs

---

## 14. Expected Result

After completion you will have:

✅ Extensible trigger architecture
✅ Clean separation of concerns
✅ Easy trigger addition from `main.rs`
✅ Wayland-ready design
✅ Compatible with future GUI daemon API

