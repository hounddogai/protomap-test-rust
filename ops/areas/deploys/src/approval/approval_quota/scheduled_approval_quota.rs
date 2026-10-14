use super::*;

use acme_kit::text;

/// Where a scheduled approval quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledApprovalQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledApprovalQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledApprovalQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled approval quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledApprovalQuota {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub amount: f64,
    pub ends_at: i64,
    pub unit: String,
    pub status: ScheduledApprovalQuotaStatus,
}

impl ScheduledApprovalQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledApprovalQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledApprovalQuotaStatus) -> bool {
        match self.status {
            ScheduledApprovalQuotaStatus::Proposed => {
                return next == ScheduledApprovalQuotaStatus::Approved;
            }
            ScheduledApprovalQuotaStatus::Approved => {
                match next {
                    ScheduledApprovalQuotaStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledApprovalQuotaStatus::Rejected => {
                return next == ScheduledApprovalQuotaStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledApprovalQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled approval quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled approval quota", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.amount + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledApprovalQuota {
        ScheduledApprovalQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            amount: self.amount,
            ends_at: self.ends_at,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a scheduled approval quota in status proposed.
pub fn new_scheduled_approval_quota(code: &str, label: &str) -> ScheduledApprovalQuota {
    ScheduledApprovalQuota {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        amount: 0.0,
        ends_at: 0,
        unit: "".to_string(),
        status: ScheduledApprovalQuotaStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_approval_quota("code-84", "label-652");
        assert_eq!(item.status, ScheduledApprovalQuotaStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_scheduled_approval_quota("code-84", "label-652");
        if item.move_to(ScheduledApprovalQuotaStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledApprovalQuotaStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_approval_quota("code-84", "label-652");
        item.status = ScheduledApprovalQuotaStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledApprovalQuotaStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
