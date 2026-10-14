use super::*;

use acme_kit::text;

/// Where a strict health check ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictHealthCheckLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl StrictHealthCheckLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for StrictHealthCheckLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict health check ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckLedger {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub description: String,
    pub active: bool,
    pub amount: f64,
    pub pinned: bool,
    pub timezone: String,
    pub owner_id: String,
    pub status: StrictHealthCheckLedgerStatus,
}

impl StrictHealthCheckLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictHealthCheckLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictHealthCheckLedgerStatus) -> bool {
        match self.status {
            StrictHealthCheckLedgerStatus::Proposed => {
                return next == StrictHealthCheckLedgerStatus::Approved;
            }
            StrictHealthCheckLedgerStatus::Approved => {
                match next {
                    StrictHealthCheckLedgerStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictHealthCheckLedgerStatus::Rejected => {
                return next == StrictHealthCheckLedgerStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictHealthCheckLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict health check ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.amount + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictHealthCheckLedger {
        StrictHealthCheckLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            description: self.description.clone(),
            active: self.active,
            amount: self.amount,
            pinned: self.pinned,
            timezone: self.timezone.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict health check ledger in status proposed.
pub fn new_strict_health_check_ledger(code: &str, label: &str) -> StrictHealthCheckLedger {
    StrictHealthCheckLedger {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        description: "".to_string(),
        active: false,
        amount: 0.0,
        pinned: false,
        timezone: "".to_string(),
        owner_id: "".to_string(),
        status: StrictHealthCheckLedgerStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_strict_health_check_ledger("code-969", "label-361");
        assert_eq!(item.status, StrictHealthCheckLedgerStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_strict_health_check_ledger("code-969", "label-361");
        if item.move_to(StrictHealthCheckLedgerStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictHealthCheckLedgerStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_strict_health_check_ledger("code-969", "label-361");
        item.status = StrictHealthCheckLedgerStatus::Withdrawn;
        assert!(
            item.move_to(StrictHealthCheckLedgerStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
