use super::*;

use acme_kit::text;

/// Where a rolling approval agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingApprovalAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingApprovalAgreementStatus {
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

impl std::fmt::Display for RollingApprovalAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling approval agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalAgreement {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub ends_at: i64,
    pub window_days: i64,
    pub quota_units: i64,
    pub batch_size: i64,
    pub cluster: String,
    pub status: RollingApprovalAgreementStatus,
}

impl RollingApprovalAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingApprovalAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingApprovalAgreementStatus) -> bool {
        match self.status {
            RollingApprovalAgreementStatus::Planned => {
                return next == RollingApprovalAgreementStatus::InProgress
                    || next == RollingApprovalAgreementStatus::Cancelled;
            }
            RollingApprovalAgreementStatus::Scheduled => {
                return next == RollingApprovalAgreementStatus::Completed;
            }
            RollingApprovalAgreementStatus::InProgress => {
                return next == RollingApprovalAgreementStatus::Completed
                    || next == RollingApprovalAgreementStatus::Cancelled;
            }
            RollingApprovalAgreementStatus::Completed => {
                match next {
                    RollingApprovalAgreementStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RollingApprovalAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling approval agreement {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.amount - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingApprovalAgreement {
        RollingApprovalAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            ends_at: self.ends_at,
            window_days: self.window_days,
            quota_units: self.quota_units,
            batch_size: self.batch_size,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rolling approval agreement in status planned.
pub fn new_rolling_approval_agreement(code: &str, name: &str) -> RollingApprovalAgreement {
    RollingApprovalAgreement {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        ends_at: 0,
        window_days: 0,
        quota_units: 0,
        batch_size: 0,
        cluster: "".to_string(),
        status: RollingApprovalAgreementStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_approval_agreement("code-591", "name-635");
        assert_eq!(item.status, RollingApprovalAgreementStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_rolling_approval_agreement("code-591", "name-635");
        if item.move_to(RollingApprovalAgreementStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingApprovalAgreementStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_approval_agreement("code-591", "name-635");
        item.status = RollingApprovalAgreementStatus::Cancelled;
        assert!(
            item.move_to(RollingApprovalAgreementStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
