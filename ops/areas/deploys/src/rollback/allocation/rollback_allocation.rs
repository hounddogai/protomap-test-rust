use super::*;

use acme_kit::text;

/// Where a rollback allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackAllocationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollbackAllocationStatus {
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

impl std::fmt::Display for RollbackAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollback allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAllocation {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub tenant_id: String,
    pub namespace: String,
    pub pinned: bool,
    pub currency: String,
    pub grace_days: i64,
    pub duration_minutes: i64,
    pub principal: String,
    pub status: RollbackAllocationStatus,
}

impl RollbackAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackAllocationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackAllocationStatus) -> bool {
        match self.status {
            RollbackAllocationStatus::New => {
                match next {
                    RollbackAllocationStatus::Running | RollbackAllocationStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackAllocationStatus::Queued => {
                return next == RollbackAllocationStatus::Running;
            }
            RollbackAllocationStatus::Running => {
                match next {
                    RollbackAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackAllocationStatus::Done => {
                return next == RollbackAllocationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rollback allocation", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.grace_days as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackAllocation {
        RollbackAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            tenant_id: self.tenant_id.clone(),
            namespace: self.namespace.clone(),
            pinned: self.pinned,
            currency: self.currency.clone(),
            grace_days: self.grace_days,
            duration_minutes: self.duration_minutes,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rollback allocation in status new.
pub fn new_rollback_allocation(code: &str, label: &str) -> RollbackAllocation {
    RollbackAllocation {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        tenant_id: "".to_string(),
        namespace: "".to_string(),
        pinned: false,
        currency: "".to_string(),
        grace_days: 0,
        duration_minutes: 0,
        principal: "".to_string(),
        status: RollbackAllocationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rollback_allocation("code-160", "label-381");
        assert_eq!(item.status, RollbackAllocationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_rollback_allocation("code-160", "label-381");
        if item.move_to(RollbackAllocationStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackAllocationStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rollback_allocation("code-160", "label-381");
        item.status = RollbackAllocationStatus::Failed;
        assert!(
            item.move_to(RollbackAllocationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
