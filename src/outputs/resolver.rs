use std::collections::HashMap;
use std::result::Result as StdResult;

/// Single source of truth for monitor/output mapping.
pub struct OutputResolver {
    outputs: Vec<String>,
}

impl OutputResolver {
    /// Detect connected outputs via `swaymsg -t get_outputs` and build the resolver.
    pub fn detect() -> StdResult<Self, Box<dyn std::error::Error>> {
        let outputs = detect_outputs()?;
        Ok(Self { outputs })
    }

    /// Build from an explicit list of output names (useful for testing or non-Sway compositors).
    pub fn from_outputs(outputs: Vec<String>) -> Self {
        Self { outputs }
    }

    /// Return the list of active outputs detected.
    pub fn outputs(&self) -> &[String] {
        &self.outputs
    }

    /// Resolve a per-output configuration map against the detected outputs.
    ///
    /// Resolution rules (per output):
    ///   1. If the map has an exact-match key → use it
    ///   2. Else if the map has a `"*"` wildcard key → use it
    ///   3. Else → skip output
    pub fn resolve_map<T: Clone>(&self, map: &HashMap<String, T>) -> HashMap<String, T> {
        let mut result = HashMap::new();

        for output in &self.outputs {
            if let Some(value) = map.get(output) {
                result.insert(output.clone(), value.clone());
            } else if let Some(wildcard) = map.get("*") {
                result.insert(output.clone(), wildcard.clone());
            }
            // Otherwise: output is not covered by this config — skip silently.
        }

        result
    }
}

/// Detect active output names by calling `swaymsg -t get_outputs` and parsing the JSON.
fn detect_outputs() -> StdResult<Vec<String>, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("swaymsg")
        .args(&["-t", "get_outputs", "-r"])
        .output();

    match output {
        Ok(cmd_output) if cmd_output.status.success() => {
            let json_str = String::from_utf8_lossy(&cmd_output.stdout);
            parse_swaymsg_outputs(&json_str)
        }
        Ok(cmd_output) => {
            let stderr = String::from_utf8_lossy(&cmd_output.stderr);
            tracing::warn!(
                "swaymsg returned non-zero status: {}. Falling back to no outputs.",
                stderr
            );
            Ok(vec![])
        }
        Err(e) => {
            tracing::warn!(
                "Could not run swaymsg to detect outputs ({}). Falling back to no outputs.",
                e
            );
            Ok(vec![])
        }
    }
}

#[derive(serde::Deserialize)]
struct SwayOutput {
    name: String,
    active: bool,
}

/// Parse the JSON output of `swaymsg -t get_outputs` and return active output names.
fn parse_swaymsg_outputs(json_str: &str) -> StdResult<Vec<String>, Box<dyn std::error::Error>> {
    let outputs: Vec<SwayOutput> = serde_json::from_str(json_str)?;
    let names = outputs
        .into_iter()
        .filter(|o| o.active)
        .map(|o| o.name)
        .collect::<Vec<_>>();

    tracing::info!("Detected outputs: {:?}", names);
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_exact_match() {
        let resolver = OutputResolver::from_outputs(vec!["HDMI-1".to_string(), "DP-1".to_string()]);

        let mut map = HashMap::new();
        map.insert("HDMI-1".to_string(), "a.png".to_string());
        map.insert("*".to_string(), "default.png".to_string());

        let resolved = resolver.resolve_map(&map);

        assert_eq!(resolved.get("HDMI-1"), Some(&"a.png".to_string()));
        assert_eq!(resolved.get("DP-1"), Some(&"default.png".to_string()));
    }

    #[test]
    fn test_resolve_wildcard_only() {
        let resolver = OutputResolver::from_outputs(vec!["HDMI-1".to_string(), "DP-1".to_string()]);

        let mut map = HashMap::new();
        map.insert("*".to_string(), "default.png".to_string());

        let resolved = resolver.resolve_map(&map);

        assert_eq!(resolved.get("HDMI-1"), Some(&"default.png".to_string()));
        assert_eq!(resolved.get("DP-1"), Some(&"default.png".to_string()));
    }

    #[test]
    fn test_resolve_missing_output_skipped() {
        let resolver = OutputResolver::from_outputs(vec!["HDMI-1".to_string()]);

        let map: HashMap<String, String> = HashMap::new();

        let resolved = resolver.resolve_map(&map);
        assert!(resolved.is_empty());
    }

    #[test]
    fn test_parse_swaymsg_outputs() {
        let json = r#"[{"name": "HDMI-A-1","active": true},{"name": "DP-1","active": false}]"#;
        let outputs = parse_swaymsg_outputs(json).unwrap();
        assert_eq!(outputs, vec!["HDMI-A-1".to_string()]);
    }
}
