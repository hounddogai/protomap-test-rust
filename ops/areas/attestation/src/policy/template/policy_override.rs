use super::*;

use acme_kit::text;

/// Where a policy override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PolicyOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PolicyOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a policy override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyOverride {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub priority: i64,
    pub threshold: f64,
    pub budget: f64,
    pub version: i64,
    pub summary: String,
    pub key_id: String,
    pub status: PolicyOverrideStatus,
}

impl PolicyOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyOverrideStatus) -> bool {
        match self.status {
            PolicyOverrideStatus::New => {
                return next == PolicyOverrideStatus::Queued || next == PolicyOverrideStatus::Failed;
            }
            PolicyOverrideStatus::Queued => {
                return next == PolicyOverrideStatus::Done;
            }
            PolicyOverrideStatus::Running => {
                return next == PolicyOverrideStatus::Done;
            }
            PolicyOverrideStatus::Done => {
                return next == PolicyOverrideStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "policy override", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.priority as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyOverride {
        PolicyOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            priority: self.priority,
            threshold: self.threshold,
            budget: self.budget,
            version: self.version,
            summary: self.summary.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a policy override in status new.
pub fn new_policy_override(code: &str, title: &str) -> PolicyOverride {
    PolicyOverride {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        priority: 0,
        threshold: 0.0,
        budget: 0.0,
        version: 0,
        summary: "".to_string(),
        key_id: "".to_string(),
        status: PolicyOverrideStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_policy_override("code-866", "title-417");
        assert_eq!(item.status, PolicyOverrideStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_policy_override("code-866", "title-417");
        if item.move_to(PolicyOverrideStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyOverrideStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_policy_override("code-866", "title-417");
        item.status = PolicyOverrideStatus::Failed;
        assert!(
            item.move_to(PolicyOverrideStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
