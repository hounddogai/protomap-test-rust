use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a weighted ttl policy bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTtlPolicyBundleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedTtlPolicyBundleStatus {
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

impl std::fmt::Display for WeightedTtlPolicyBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted ttl policy bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyBundle {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub archived: bool,
    pub ceiling: f64,
    pub key_id: String,
    pub status: WeightedTtlPolicyBundleStatus,
}

impl WeightedTtlPolicyBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTtlPolicyBundleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTtlPolicyBundleStatus) -> bool {
        match self.status {
            WeightedTtlPolicyBundleStatus::Planned => {
                return next == WeightedTtlPolicyBundleStatus::Cancelled;
            }
            WeightedTtlPolicyBundleStatus::Scheduled => {
                match next {
                    WeightedTtlPolicyBundleStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTtlPolicyBundleStatus::InProgress => {
                match next {
                    WeightedTtlPolicyBundleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTtlPolicyBundleStatus::Completed => {
                return next == WeightedTtlPolicyBundleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedTtlPolicyBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted ttl policy bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
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
        self.ceiling
    }

    pub fn copy(&self) -> WeightedTtlPolicyBundle {
        WeightedTtlPolicyBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            archived: self.archived,
            ceiling: self.ceiling,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted ttl policy bundle in status planned.
pub fn new_weighted_ttl_policy_bundle(code: &str, label: &str) -> WeightedTtlPolicyBundle {
    WeightedTtlPolicyBundle {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        archived: false,
        ceiling: 0.0,
        key_id: "".to_string(),
        status: WeightedTtlPolicyBundleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_ttl_policy_bundle("code-869", "label-952");
        assert_eq!(item.status, WeightedTtlPolicyBundleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_weighted_ttl_policy_bundle("code-869", "label-952");
        if item.move_to(WeightedTtlPolicyBundleStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedTtlPolicyBundleStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_ttl_policy_bundle("code-869", "label-952");
        item.status = WeightedTtlPolicyBundleStatus::Cancelled;
        assert!(
            item.move_to(WeightedTtlPolicyBundleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
