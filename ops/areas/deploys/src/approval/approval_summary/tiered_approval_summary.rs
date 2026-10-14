use super::*;

use acme_kit::text;

/// Where a tiered approval summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredApprovalSummaryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredApprovalSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredApprovalSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered approval summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalSummary {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub batch_size: i64,
    pub ratio: f64,
    pub grace_days: i64,
    pub owner_id: String,
    pub amount: f64,
    pub capacity: i64,
    pub status: TieredApprovalSummaryStatus,
}

impl TieredApprovalSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredApprovalSummaryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredApprovalSummaryStatus) -> bool {
        match self.status {
            TieredApprovalSummaryStatus::Proposed => {
                return next == TieredApprovalSummaryStatus::Approved;
            }
            TieredApprovalSummaryStatus::Approved => {
                return next == TieredApprovalSummaryStatus::Rejected || next == TieredApprovalSummaryStatus::Withdrawn;
            }
            TieredApprovalSummaryStatus::Rejected => {
                return next == TieredApprovalSummaryStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredApprovalSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered approval summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.grace_days as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredApprovalSummary {
        TieredApprovalSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            batch_size: self.batch_size,
            ratio: self.ratio,
            grace_days: self.grace_days,
            owner_id: self.owner_id.clone(),
            amount: self.amount,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered approval summary in status proposed.
pub fn new_tiered_approval_summary(code: &str, name: &str) -> TieredApprovalSummary {
    TieredApprovalSummary {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        batch_size: 0,
        ratio: 0.0,
        grace_days: 0,
        owner_id: "".to_string(),
        amount: 0.0,
        capacity: 0,
        status: TieredApprovalSummaryStatus::Proposed,
    }
}
