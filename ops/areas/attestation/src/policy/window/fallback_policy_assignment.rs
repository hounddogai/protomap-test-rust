use super::*;

use acme_kit::text;

/// Where a fallback policy assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPolicyAssignmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackPolicyAssignmentStatus {
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

impl std::fmt::Display for FallbackPolicyAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback policy assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPolicyAssignment {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub phone: String,
    pub cluster: String,
    pub source: String,
    pub floor_value: f64,
    pub ratio: f64,
    pub attempts: i64,
    pub status: FallbackPolicyAssignmentStatus,
}

impl FallbackPolicyAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPolicyAssignmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPolicyAssignmentStatus) -> bool {
        match self.status {
            FallbackPolicyAssignmentStatus::New => {
                match next {
                    FallbackPolicyAssignmentStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPolicyAssignmentStatus::Queued => {
                return next == FallbackPolicyAssignmentStatus::Done || next == FallbackPolicyAssignmentStatus::Failed;
            }
            FallbackPolicyAssignmentStatus::Running => {
                return next == FallbackPolicyAssignmentStatus::Done || next == FallbackPolicyAssignmentStatus::Failed;
            }
            FallbackPolicyAssignmentStatus::Done => {
                return next == FallbackPolicyAssignmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPolicyAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback policy assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.attempts as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPolicyAssignment {
        FallbackPolicyAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            phone: self.phone.clone(),
            cluster: self.cluster.clone(),
            source: self.source.clone(),
            floor_value: self.floor_value,
            ratio: self.ratio,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a fallback policy assignment in status new.
pub fn new_fallback_policy_assignment(code: &str, label: &str) -> FallbackPolicyAssignment {
    FallbackPolicyAssignment {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        phone: "".to_string(),
        cluster: "".to_string(),
        source: "".to_string(),
        floor_value: 0.0,
        ratio: 0.0,
        attempts: 0,
        status: FallbackPolicyAssignmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_policy_assignment("code-490", "label-178");
        assert_eq!(item.status, FallbackPolicyAssignmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_fallback_policy_assignment("code-490", "label-178");
        if item.move_to(FallbackPolicyAssignmentStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPolicyAssignmentStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_policy_assignment("code-490", "label-178");
        item.status = FallbackPolicyAssignmentStatus::Failed;
        assert!(
            item.move_to(FallbackPolicyAssignmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
