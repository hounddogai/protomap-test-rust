use super::*;

use acme_kit::text;

/// Where a scheduled canary policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledCanaryPolicyStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledCanaryPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ScheduledCanaryPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled canary policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanaryPolicy {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub notes: String,
    pub unit: String,
    pub floor_value: f64,
    pub category: String,
    pub checksum: String,
    pub phone: String,
    pub status: ScheduledCanaryPolicyStatus,
}

impl ScheduledCanaryPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledCanaryPolicyStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledCanaryPolicyStatus) -> bool {
        match self.status {
            ScheduledCanaryPolicyStatus::Open => {
                match next {
                    ScheduledCanaryPolicyStatus::Resolved | ScheduledCanaryPolicyStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledCanaryPolicyStatus::InReview => {
                return next == ScheduledCanaryPolicyStatus::Resolved || next == ScheduledCanaryPolicyStatus::Archived;
            }
            ScheduledCanaryPolicyStatus::Resolved => {
                return next == ScheduledCanaryPolicyStatus::Reopened || next == ScheduledCanaryPolicyStatus::Archived;
            }
            ScheduledCanaryPolicyStatus::Reopened => {
                match next {
                    ScheduledCanaryPolicyStatus::Archived => {
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

    pub fn move_to(&mut self, next: ScheduledCanaryPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled canary policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.floor_value
    }

    pub fn copy(&self) -> ScheduledCanaryPolicy {
        ScheduledCanaryPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            notes: self.notes.clone(),
            unit: self.unit.clone(),
            floor_value: self.floor_value,
            category: self.category.clone(),
            checksum: self.checksum.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a scheduled canary policy in status open.
pub fn new_scheduled_canary_policy(code: &str, name: &str) -> ScheduledCanaryPolicy {
    ScheduledCanaryPolicy {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        notes: "".to_string(),
        unit: "".to_string(),
        floor_value: 0.0,
        category: "".to_string(),
        checksum: "".to_string(),
        phone: "".to_string(),
        status: ScheduledCanaryPolicyStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_canary_policy("code-750", "name-707");
        assert_eq!(item.status, ScheduledCanaryPolicyStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_scheduled_canary_policy("code-750", "name-707");
        if item.move_to(ScheduledCanaryPolicyStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledCanaryPolicyStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_canary_policy("code-750", "name-707");
        item.status = ScheduledCanaryPolicyStatus::Archived;
        assert!(
            item.move_to(ScheduledCanaryPolicyStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
