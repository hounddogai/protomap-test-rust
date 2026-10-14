use super::*;

use acme_kit::text;

/// Where a pending policy schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPolicyScheduleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PendingPolicyScheduleStatus {
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

impl std::fmt::Display for PendingPolicyScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending policy schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicySchedule {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub grace_days: i64,
    pub batch_size: i64,
    pub minimum: f64,
    pub score: f64,
    pub contact_email: String,
    pub region: String,
    pub external_ref: String,
    pub tenant_id: String,
    pub status: PendingPolicyScheduleStatus,
}

impl PendingPolicySchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPolicyScheduleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPolicyScheduleStatus) -> bool {
        match self.status {
            PendingPolicyScheduleStatus::Planned => {
                match next {
                    PendingPolicyScheduleStatus::Scheduled | PendingPolicyScheduleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPolicyScheduleStatus::Scheduled => {
                match next {
                    PendingPolicyScheduleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPolicyScheduleStatus::InProgress => {
                return next == PendingPolicyScheduleStatus::Cancelled;
            }
            PendingPolicyScheduleStatus::Completed => {
                match next {
                    PendingPolicyScheduleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PendingPolicyScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending policy schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.minimum + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPolicySchedule {
        PendingPolicySchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            grace_days: self.grace_days,
            batch_size: self.batch_size,
            minimum: self.minimum,
            score: self.score,
            contact_email: self.contact_email.clone(),
            region: self.region.clone(),
            external_ref: self.external_ref.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a pending policy schedule in status planned.
pub fn new_pending_policy_schedule(code: &str, label: &str) -> PendingPolicySchedule {
    PendingPolicySchedule {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        grace_days: 0,
        batch_size: 0,
        minimum: 0.0,
        score: 0.0,
        contact_email: "".to_string(),
        region: "".to_string(),
        external_ref: "".to_string(),
        tenant_id: "".to_string(),
        status: PendingPolicyScheduleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_pending_policy_schedule("code-183", "label-762");
        assert_eq!(item.status, PendingPolicyScheduleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_pending_policy_schedule("code-183", "label-762");
        if item.move_to(PendingPolicyScheduleStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPolicyScheduleStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_pending_policy_schedule("code-183", "label-762");
        item.status = PendingPolicyScheduleStatus::Cancelled;
        assert!(
            item.move_to(PendingPolicyScheduleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
