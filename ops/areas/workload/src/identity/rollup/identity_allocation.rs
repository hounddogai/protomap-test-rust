use super::*;

use acme_kit::text;

/// Where a identity allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IdentityAllocationStatus {
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

impl std::fmt::Display for IdentityAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAllocation {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub scope: String,
    pub summary: String,
    pub batch_size: i64,
    pub duration_minutes: i64,
    pub fingerprint: String,
    pub status: IdentityAllocationStatus,
}

impl IdentityAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityAllocationStatus) -> bool {
        match self.status {
            IdentityAllocationStatus::Planned => {
                match next {
                    IdentityAllocationStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityAllocationStatus::Scheduled => {
                match next {
                    IdentityAllocationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityAllocationStatus::InProgress => {
                match next {
                    IdentityAllocationStatus::Completed | IdentityAllocationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityAllocationStatus::Completed => {
                return next == IdentityAllocationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "identity allocation", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.batch_size as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityAllocation {
        IdentityAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            scope: self.scope.clone(),
            summary: self.summary.clone(),
            batch_size: self.batch_size,
            duration_minutes: self.duration_minutes,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a identity allocation in status planned.
pub fn new_identity_allocation(code: &str, label: &str) -> IdentityAllocation {
    IdentityAllocation {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        scope: "".to_string(),
        summary: "".to_string(),
        batch_size: 0,
        duration_minutes: 0,
        fingerprint: "".to_string(),
        status: IdentityAllocationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_identity_allocation("code-282", "label-697");
        assert_eq!(item.status, IdentityAllocationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_identity_allocation("code-282", "label-697");
        if item.move_to(IdentityAllocationStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityAllocationStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_identity_allocation("code-282", "label-697");
        item.status = IdentityAllocationStatus::Cancelled;
        assert!(
            item.move_to(IdentityAllocationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
