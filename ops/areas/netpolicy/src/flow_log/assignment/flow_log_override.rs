use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a flow log override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FlowLogOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for FlowLogOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a flow log override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogOverride {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub tags: Vec<String>,
    pub namespace: String,
    pub ends_at: i64,
    pub floor_value: f64,
    pub category: String,
    pub status: FlowLogOverrideStatus,
}

impl FlowLogOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogOverrideStatus) -> bool {
        match self.status {
            FlowLogOverrideStatus::Planned => {
                return next == FlowLogOverrideStatus::Cancelled;
            }
            FlowLogOverrideStatus::Scheduled => {
                match next {
                    FlowLogOverrideStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogOverrideStatus::InProgress => {
                return next == FlowLogOverrideStatus::Cancelled;
            }
            FlowLogOverrideStatus::Completed => {
                match next {
                    FlowLogOverrideStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: FlowLogOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn effective_value(&self) -> f64 {
        (self.floor_value * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogOverride {
        FlowLogOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            tags: self.tags.clone(),
            namespace: self.namespace.clone(),
            ends_at: self.ends_at,
            floor_value: self.floor_value,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a flow log override in status planned.
pub fn new_flow_log_override(code: &str, title: &str) -> FlowLogOverride {
    FlowLogOverride {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        tags: Vec::new(),
        namespace: "".to_string(),
        ends_at: 0,
        floor_value: 0.0,
        category: "".to_string(),
        status: FlowLogOverrideStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_flow_log_override("code-619", "title-389");
        assert_eq!(item.status, FlowLogOverrideStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_flow_log_override("code-619", "title-389");
        if item.move_to(FlowLogOverrideStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogOverrideStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_flow_log_override("code-619", "title-389");
        item.status = FlowLogOverrideStatus::Cancelled;
        assert!(
            item.move_to(FlowLogOverrideStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
