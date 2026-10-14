use super::*;

use acme_kit::text;

/// Where a adaptive budget burn threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveBudgetBurnThresholdStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveBudgetBurnThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveBudgetBurnThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive budget burn threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBudgetBurnThreshold {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub pinned: bool,
    pub country: String,
    pub visible: bool,
    pub score: f64,
    pub status: AdaptiveBudgetBurnThresholdStatus,
}

impl AdaptiveBudgetBurnThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveBudgetBurnThresholdStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveBudgetBurnThresholdStatus) -> bool {
        match self.status {
            AdaptiveBudgetBurnThresholdStatus::Proposed => {
                return next == AdaptiveBudgetBurnThresholdStatus::Withdrawn;
            }
            AdaptiveBudgetBurnThresholdStatus::Approved => {
                return next == AdaptiveBudgetBurnThresholdStatus::Rejected;
            }
            AdaptiveBudgetBurnThresholdStatus::Rejected => {
                match next {
                    AdaptiveBudgetBurnThresholdStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AdaptiveBudgetBurnThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive budget burn threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive budget burn threshold", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> AdaptiveBudgetBurnThreshold {
        AdaptiveBudgetBurnThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            pinned: self.pinned,
            country: self.country.clone(),
            visible: self.visible,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a adaptive budget burn threshold in status proposed.
pub fn new_adaptive_budget_burn_threshold(code: &str, label: &str) -> AdaptiveBudgetBurnThreshold {
    AdaptiveBudgetBurnThreshold {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        pinned: false,
        country: "".to_string(),
        visible: false,
        score: 0.0,
        status: AdaptiveBudgetBurnThresholdStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_budget_burn_threshold("code-273", "label-157");
        assert_eq!(item.status, AdaptiveBudgetBurnThresholdStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_adaptive_budget_burn_threshold("code-273", "label-157");
        if item.move_to(AdaptiveBudgetBurnThresholdStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveBudgetBurnThresholdStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_budget_burn_threshold("code-273", "label-157");
        item.status = AdaptiveBudgetBurnThresholdStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveBudgetBurnThresholdStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
