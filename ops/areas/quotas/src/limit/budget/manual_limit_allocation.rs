use super::*;

use acme_kit::text;

/// Where a manual limit allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualLimitAllocationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualLimitAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ManualLimitAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual limit allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitAllocation {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub amount: f64,
    pub unit: String,
    pub visible: bool,
    pub attempts: i64,
    pub revision: i64,
    pub cost: f64,
    pub maximum: f64,
    pub threshold: f64,
    pub status: ManualLimitAllocationStatus,
}

impl ManualLimitAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualLimitAllocationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualLimitAllocationStatus) -> bool {
        match self.status {
            ManualLimitAllocationStatus::Open => {
                return next == ManualLimitAllocationStatus::InReview || next == ManualLimitAllocationStatus::Resolved;
            }
            ManualLimitAllocationStatus::InReview => {
                return next == ManualLimitAllocationStatus::Resolved;
            }
            ManualLimitAllocationStatus::Resolved => {
                return next == ManualLimitAllocationStatus::Archived;
            }
            ManualLimitAllocationStatus::Reopened => {
                return next == ManualLimitAllocationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualLimitAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual limit allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.cost + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualLimitAllocation {
        ManualLimitAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            amount: self.amount,
            unit: self.unit.clone(),
            visible: self.visible,
            attempts: self.attempts,
            revision: self.revision,
            cost: self.cost,
            maximum: self.maximum,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a manual limit allocation in status open.
pub fn new_manual_limit_allocation(code: &str, title: &str) -> ManualLimitAllocation {
    ManualLimitAllocation {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        amount: 0.0,
        unit: "".to_string(),
        visible: false,
        attempts: 0,
        revision: 0,
        cost: 0.0,
        maximum: 0.0,
        threshold: 0.0,
        status: ManualLimitAllocationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_limit_allocation("code-636", "title-545");
        assert_eq!(item.status, ManualLimitAllocationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_manual_limit_allocation("code-636", "title-545");
        if item.move_to(ManualLimitAllocationStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualLimitAllocationStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_limit_allocation("code-636", "title-545");
        item.status = ManualLimitAllocationStatus::Archived;
        assert!(
            item.move_to(ManualLimitAllocationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
