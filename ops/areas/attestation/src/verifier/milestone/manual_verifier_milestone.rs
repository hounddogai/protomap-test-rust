use super::*;

use acme_kit::text;

/// Where a manual verifier milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualVerifierMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualVerifierMilestoneStatus {
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

impl std::fmt::Display for ManualVerifierMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual verifier milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierMilestone {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub grace_days: i64,
    pub ceiling: f64,
    pub active: bool,
    pub region: String,
    pub status: ManualVerifierMilestoneStatus,
}

impl ManualVerifierMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualVerifierMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualVerifierMilestoneStatus) -> bool {
        match self.status {
            ManualVerifierMilestoneStatus::New => {
                return next == ManualVerifierMilestoneStatus::Running;
            }
            ManualVerifierMilestoneStatus::Queued => {
                return next == ManualVerifierMilestoneStatus::Failed;
            }
            ManualVerifierMilestoneStatus::Running => {
                return next == ManualVerifierMilestoneStatus::Done || next == ManualVerifierMilestoneStatus::Failed;
            }
            ManualVerifierMilestoneStatus::Done => {
                return next == ManualVerifierMilestoneStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualVerifierMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual verifier milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual verifier milestone", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ceiling + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualVerifierMilestone {
        ManualVerifierMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            grace_days: self.grace_days,
            ceiling: self.ceiling,
            active: self.active,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual verifier milestone in status new.
pub fn new_manual_verifier_milestone(code: &str, label: &str) -> ManualVerifierMilestone {
    ManualVerifierMilestone {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        grace_days: 0,
        ceiling: 0.0,
        active: false,
        region: "".to_string(),
        status: ManualVerifierMilestoneStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_verifier_milestone("code-117", "label-882");
        assert_eq!(item.status, ManualVerifierMilestoneStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_manual_verifier_milestone("code-117", "label-882");
        if item.move_to(ManualVerifierMilestoneStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualVerifierMilestoneStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_verifier_milestone("code-117", "label-882");
        item.status = ManualVerifierMilestoneStatus::Failed;
        assert!(
            item.move_to(ManualVerifierMilestoneStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
