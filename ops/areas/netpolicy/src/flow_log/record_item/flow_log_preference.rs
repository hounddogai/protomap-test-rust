use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a flow log preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FlowLogPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for FlowLogPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log preference as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPreference {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub external_ref: String,
    pub country: String,
    pub margin: f64,
    pub cluster: String,
    pub status: FlowLogPreferenceStatus,
}

impl FlowLogPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogPreferenceStatus) -> bool {
        match self.status {
            FlowLogPreferenceStatus::Idle => {
                return next == FlowLogPreferenceStatus::Draining;
            }
            FlowLogPreferenceStatus::Warming => {
                return next == FlowLogPreferenceStatus::Stopped;
            }
            FlowLogPreferenceStatus::Ready => {
                return next == FlowLogPreferenceStatus::Draining;
            }
            FlowLogPreferenceStatus::Draining => {
                match next {
                    FlowLogPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FlowLogPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> FlowLogPreference {
        FlowLogPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            external_ref: self.external_ref.clone(),
            country: self.country.clone(),
            margin: self.margin,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a flow log preference in status idle.
pub fn new_flow_log_preference(code: &str, name: &str) -> FlowLogPreference {
    FlowLogPreference {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        external_ref: "".to_string(),
        country: "".to_string(),
        margin: 0.0,
        cluster: "".to_string(),
        status: FlowLogPreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_flow_log_preference("code-532", "name-28");
        assert_eq!(item.status, FlowLogPreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_flow_log_preference("code-532", "name-28");
        if item.move_to(FlowLogPreferenceStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogPreferenceStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_flow_log_preference("code-532", "name-28");
        item.status = FlowLogPreferenceStatus::Stopped;
        assert!(
            item.move_to(FlowLogPreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
