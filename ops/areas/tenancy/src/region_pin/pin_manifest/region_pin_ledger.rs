use super::*;

use acme_kit::text;

/// Where a region pin ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinLedgerStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionPinLedgerStatus {
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

impl std::fmt::Display for RegionPinLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One region pin ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinLedger {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub phone: String,
    pub grace_days: i64,
    pub archived: bool,
    pub currency: String,
    pub status: RegionPinLedgerStatus,
}

impl RegionPinLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinLedgerStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinLedgerStatus) -> bool {
        match self.status {
            RegionPinLedgerStatus::Planned => {
                return next == RegionPinLedgerStatus::Cancelled;
            }
            RegionPinLedgerStatus::Scheduled => {
                return next == RegionPinLedgerStatus::InProgress;
            }
            RegionPinLedgerStatus::InProgress => {
                match next {
                    RegionPinLedgerStatus::Completed | RegionPinLedgerStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinLedgerStatus::Completed => {
                match next {
                    RegionPinLedgerStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RegionPinLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.weight + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinLedger {
        RegionPinLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            phone: self.phone.clone(),
            grace_days: self.grace_days,
            archived: self.archived,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a region pin ledger in status planned.
pub fn new_region_pin_ledger(code: &str, label: &str) -> RegionPinLedger {
    RegionPinLedger {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        phone: "".to_string(),
        grace_days: 0,
        archived: false,
        currency: "".to_string(),
        status: RegionPinLedgerStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_region_pin_ledger("code-19", "label-43");
        assert_eq!(item.status, RegionPinLedgerStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_region_pin_ledger("code-19", "label-43");
        if item.move_to(RegionPinLedgerStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinLedgerStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_region_pin_ledger("code-19", "label-43");
        item.status = RegionPinLedgerStatus::Cancelled;
        assert!(
            item.move_to(RegionPinLedgerStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
