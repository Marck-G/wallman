Perfect — this is an important step because **multi-monitor support changes how triggers reason about decisions**.
Right now triggers return a single wallpaper decision, but your configuration already implies:

* Multiple outputs (`HDMI-1`, `DP-1`, etc.)
* Wildcard support (`*`)
* Different triggers possibly affecting different monitors simultaneously.

Below is a **detailed implementation plan** describing:

1. How multi-monitor support must work internally
2. Required architectural changes
3. How triggers must be modified
4. How `*` fallback resolution works
5. Concrete implementation steps

---

# ✅ Wallman — Multi-Monitor Support Implementation Plan

---

# 1. Goal

Enable Wallman to:

* Apply wallpapers per output (monitor)
* Support configuration maps:

  * `[background.HDMI-1]`
  * `[background.*]`
* Allow triggers to evaluate **multiple outputs at once**
* Maintain compatibility with future Wayland layer backend

---

# 2. Core Design Change (Critical)

## ❌ Current model (implicit)

Triggers return:

```rust
TriggerResult {
    output,
    image_path
}
```

This assumes **one monitor at a time**.

---

## ✅ New model (required)

Triggers must return decisions for **multiple outputs**.

### New result type:

```rust
pub struct TriggerResult {
    pub changes: Vec<OutputChange>,
}
```

```rust
pub struct OutputChange {
    pub output: String,
    pub image_path: String,
}
```

---

### Why this is necessary

Because:

* One trigger evaluation may affect ALL monitors.
* `*` must expand dynamically.
* Avoid running trigger logic once per monitor.

---

# 3. Output Resolution Layer (NEW MODULE)

Create:

```text
src/outputs/
    resolver.rs
```

This module becomes the **single source of truth** for monitor mapping.

---

## Responsibilities

The resolver must:

1. Detect available outputs (via swaymsg for now).
2. Resolve configuration using:

   * explicit output
   * wildcard `*`
3. Produce final per-output configuration.

---

## OutputResolver structure

```rust
pub struct OutputResolver {
    outputs: Vec<String>,
}
```

---

## Required Function

```rust
fn resolve_map<T: Clone>(
    map: &HashMap<String, T>,
    outputs: &[String],
) -> HashMap<String, T>
```

---

### Resolution Rules

For each detected output:

1. If config contains exact match → use it
2. Else if `"*"` exists → use wildcard
3. Else → ignore output

---

### Example

Config:

```toml
[background.HDMI-1]
image="a.png"

[background.*]
image="default.png"
```

Detected outputs:

```text
HDMI-1
DP-1
```

Resolved result:

```text
HDMI-1 → a.png
DP-1   → default.png
```

---

# 4. Output Detection (MVP)

Create helper:

```rust
fn detect_outputs() -> Result<Vec<String>>
```

Implementation (MVP):

* Call `swaymsg -t get_outputs`
* Parse JSON
* Extract active output names

Later this becomes Wayland-native.

---

# 5. Changes Required in Trigger Trait

## OLD

```rust
fn evaluate(&mut self) -> Result<Option<TriggerResult>>;
```

## NEW (same signature, different semantics)

Triggers now:

✅ evaluate ALL outputs
✅ return batch changes

---

# 6. Access Pattern Inside Triggers

Every trigger must now:

1. Clone config
2. Detect outputs
3. Resolve hashmap
4. Produce results per output

---

### Standard workflow inside a trigger

```text
lock AppState briefly
        ↓
clone config
        ↓
detect outputs
        ↓
resolve wildcard map
        ↓
compute image per output
        ↓
return Vec<OutputChange>
```

---

# 7. Static Trigger Modification

### Before

Returned single wallpaper.

### Now

Steps:

1. Read `config.background`
2. Resolve outputs using resolver
3. Produce `OutputChange` for each output

Example result:

```rust
vec![
  { HDMI-1, wall1.png },
  { DP-1, wall2.png }
]
```

---

# 8. DayTime Trigger Modification

### New Logic

For EACH resolved output:

1. Determine day/night
2. Select correct image
3. Compare with last applied state per output

---

### Internal State MUST change

```rust
pub struct DayTimeTrigger {
    last_state: HashMap<String, bool>,
}
```

Key = output name.

---

# 9. Weather Trigger Modification

Same principle.

---

### Internal state:

```rust
pub struct WeatherTrigger {
    last_weather: HashMap<String, WeatherStates>,
}
```

Even if weather source is same, outputs may differ in config mapping.

---

# 10. Wallpaper Apply Layer Update

Modify wallpaper module:

```rust
pub fn apply(result: TriggerResult) -> Result<()> {
    for change in result.changes {
        apply_to_output(change)?;
    }
}
```

---

### apply_to_output()

For swaybg MVP:

* spawn swaybg per output
* track running processes (future improvement)

---

# 11. TriggerManager Changes

Manager logic remains mostly unchanged.

Only difference:

```rust
if let Some(result) = trigger.evaluate()? {
    wallpaper::apply(result)?;
}
```

Now applies batch updates.

---

# 12. Edge Cases (Must Handle)

### Missing image

Skip output, log warning.

### Output disconnected

Ignore silently.

### Wildcard only config

Apply to all outputs.

### Hotplug monitors (future)

Resolver re-runs every evaluation.

---

# 13. Implementation Order (Strict)

### Phase 1 — Infrastructure

* Create outputs module
* Implement output detection
* Implement wildcard resolver

### Phase 2 — Core Changes

* Update TriggerResult
* Update wallpaper apply logic

### Phase 3 — Trigger Migration

* Update StaticTrigger
* Update DayTimeTrigger
* Update WeatherTrigger

### Phase 4 — Validation

* Multi-monitor testing
* Mixed explicit + wildcard configs
* Monitor disconnect/reconnect

---

# 14. Expected Final Architecture

```text
Triggers
   ↓
Output Resolver
   ↓
Batch Output Decisions
   ↓
Wallpaper Backend (swaybg / wayland layer)
```

---

# 15. Important Architectural Benefit

After this change:

✅ Adding new triggers requires ZERO multi-monitor logic rewrite
✅ Wayland migration becomes backend-only
✅ GUI preview per monitor becomes trivial
✅ Hot reload becomes deterministic


