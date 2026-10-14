use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched policy template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPolicyTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedPolicyTemplateStatus {
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

impl std::fmt::Display for BatchedPolicyTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched policy template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTemplate {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub capacity: i64,
    pub key_id: String,
    pub attributes: BTreeMap<String, String>,
    pub port: i64,
    pub duration_minutes: i64,
    pub burst: i64,
    pub status: BatchedPolicyTemplateStatus,
}

impl BatchedPolicyTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPolicyTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPolicyTemplateStatus) -> bool {
        match self.status {
            BatchedPolicyTemplateStatus::Planned => {
                match next {
                    BatchedPolicyTemplateStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPolicyTemplateStatus::Scheduled => {
                match next {
                    BatchedPolicyTemplateStatus::InProgress | BatchedPolicyTemplateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPolicyTemplateStatus::InProgress => {
                return next == BatchedPolicyTemplateStatus::Completed;
            }
            BatchedPolicyTemplateStatus::Completed => {
                return next == BatchedPolicyTemplateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedPolicyTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched policy template {} cannot move from {} to {}",
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

    pub fn adjusted_load(&self) -> f64 {
        ((self.port as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedPolicyTemplate {
        BatchedPolicyTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            capacity: self.capacity,
            key_id: self.key_id.clone(),
            attributes: self.attributes.clone(),
            port: self.port,
            duration_minutes: self.duration_minutes,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched policy template in status planned.
pub fn new_batched_policy_template(code: &str, label: &str) -> BatchedPolicyTemplate {
    BatchedPolicyTemplate {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        capacity: 0,
        key_id: "".to_string(),
        attributes: BTreeMap::new(),
        port: 0,
        duration_minutes: 0,
        burst: 0,
        status: BatchedPolicyTemplateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_policy_template("code-80", "label-452");
        assert_eq!(item.status, BatchedPolicyTemplateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_batched_policy_template("code-80", "label-452");
        if item.move_to(BatchedPolicyTemplateStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedPolicyTemplateStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_policy_template("code-80", "label-452");
        item.status = BatchedPolicyTemplateStatus::Cancelled;
        assert!(
            item.move_to(BatchedPolicyTemplateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
