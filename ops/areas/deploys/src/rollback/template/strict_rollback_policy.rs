use super::*;

use acme_kit::text;

/// Where a strict rollback policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRollbackPolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictRollbackPolicyStatus {
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

impl std::fmt::Display for StrictRollbackPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict rollback policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRollbackPolicy {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub description: String,
    pub summary: String,
    pub score: f64,
    pub status: StrictRollbackPolicyStatus,
}

impl StrictRollbackPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRollbackPolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRollbackPolicyStatus) -> bool {
        match self.status {
            StrictRollbackPolicyStatus::New => {
                return next == StrictRollbackPolicyStatus::Queued || next == StrictRollbackPolicyStatus::Running;
            }
            StrictRollbackPolicyStatus::Queued => {
                match next {
                    StrictRollbackPolicyStatus::Running | StrictRollbackPolicyStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRollbackPolicyStatus::Running => {
                match next {
                    StrictRollbackPolicyStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRollbackPolicyStatus::Done => {
                match next {
                    StrictRollbackPolicyStatus::Failed => {
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

    pub fn move_to(&mut self, next: StrictRollbackPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict rollback policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> StrictRollbackPolicy {
        StrictRollbackPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            description: self.description.clone(),
            summary: self.summary.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict rollback policy in status new.
pub fn new_strict_rollback_policy(code: &str, name: &str) -> StrictRollbackPolicy {
    StrictRollbackPolicy {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        description: "".to_string(),
        summary: "".to_string(),
        score: 0.0,
        status: StrictRollbackPolicyStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_rollback_policy("code-595", "name-849");
        assert_eq!(item.status, StrictRollbackPolicyStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_strict_rollback_policy("code-595", "name-849");
        if item.move_to(StrictRollbackPolicyStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictRollbackPolicyStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_rollback_policy("code-595", "name-849");
        item.status = StrictRollbackPolicyStatus::Failed;
        assert!(
            item.move_to(StrictRollbackPolicyStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
