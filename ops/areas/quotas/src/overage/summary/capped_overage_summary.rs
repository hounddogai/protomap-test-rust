use super::*;

use acme_kit::text;

/// Where a capped overage summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedOverageSummaryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedOverageSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedOverageSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped overage summary as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageSummary {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub quantity: i64,
    pub ceiling: f64,
    pub visible: bool,
    pub cost: f64,
    pub status: CappedOverageSummaryStatus,
}

impl CappedOverageSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedOverageSummaryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedOverageSummaryStatus) -> bool {
        match self.status {
            CappedOverageSummaryStatus::Proposed => {
                match next {
                    CappedOverageSummaryStatus::Approved | CappedOverageSummaryStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedOverageSummaryStatus::Approved => {
                return next == CappedOverageSummaryStatus::Withdrawn;
            }
            CappedOverageSummaryStatus::Rejected => {
                match next {
                    CappedOverageSummaryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: CappedOverageSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped overage summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped overage summary", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.cost - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedOverageSummary {
        CappedOverageSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            quantity: self.quantity,
            ceiling: self.ceiling,
            visible: self.visible,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped overage summary in status proposed.
pub fn new_capped_overage_summary(code: &str, label: &str) -> CappedOverageSummary {
    CappedOverageSummary {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        quantity: 0,
        ceiling: 0.0,
        visible: false,
        cost: 0.0,
        status: CappedOverageSummaryStatus::Proposed,
    }
}
