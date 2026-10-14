use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a retention plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionPlanStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RetentionPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RetentionPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPlan {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub floor_value: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: RetentionPlanStatus,
}

impl RetentionPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionPlanStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionPlanStatus) -> bool {
        match self.status {
            RetentionPlanStatus::Proposed => {
                match next {
                    RetentionPlanStatus::Approved | RetentionPlanStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionPlanStatus::Approved => {
                return next == RetentionPlanStatus::Rejected || next == RetentionPlanStatus::Withdrawn;
            }
            RetentionPlanStatus::Rejected => {
                return next == RetentionPlanStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn effective_load(&self) -> f64 {
        ((self.floor_value + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionPlan {
        RetentionPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            floor_value: self.floor_value,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a retention plan in status proposed.
pub fn new_retention_plan(code: &str, label: &str) -> RetentionPlan {
    RetentionPlan {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        floor_value: 0.0,
        attributes: BTreeMap::new(),
        status: RetentionPlanStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_retention_plan("code-924", "label-89");
        assert_eq!(item.status, RetentionPlanStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_retention_plan("code-924", "label-89");
        if item.move_to(RetentionPlanStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionPlanStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_retention_plan("code-924", "label-89");
        item.status = RetentionPlanStatus::Withdrawn;
        assert!(
            item.move_to(RetentionPlanStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
