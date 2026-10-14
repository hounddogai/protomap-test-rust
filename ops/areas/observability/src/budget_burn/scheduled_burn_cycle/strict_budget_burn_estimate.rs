use super::*;

use acme_kit::text;

/// Where a strict budget burn estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictBudgetBurnEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictBudgetBurnEstimateStatus {
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

impl std::fmt::Display for StrictBudgetBurnEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict budget burn estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnEstimate {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub expires_at: i64,
    pub contact_email: String,
    pub ceiling: f64,
    pub archived: bool,
    pub threshold: f64,
    pub country: String,
    pub visible: bool,
    pub floor_value: f64,
    pub status: StrictBudgetBurnEstimateStatus,
}

impl StrictBudgetBurnEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictBudgetBurnEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictBudgetBurnEstimateStatus) -> bool {
        match self.status {
            StrictBudgetBurnEstimateStatus::Open => {
                return next == StrictBudgetBurnEstimateStatus::InReview;
            }
            StrictBudgetBurnEstimateStatus::InReview => {
                match next {
                    StrictBudgetBurnEstimateStatus::Resolved | StrictBudgetBurnEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictBudgetBurnEstimateStatus::Resolved => {
                match next {
                    StrictBudgetBurnEstimateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictBudgetBurnEstimateStatus::Reopened => {
                match next {
                    StrictBudgetBurnEstimateStatus::Archived => {
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

    pub fn move_to(&mut self, next: StrictBudgetBurnEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict budget burn estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.expires_at as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictBudgetBurnEstimate {
        StrictBudgetBurnEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            expires_at: self.expires_at,
            contact_email: self.contact_email.clone(),
            ceiling: self.ceiling,
            archived: self.archived,
            threshold: self.threshold,
            country: self.country.clone(),
            visible: self.visible,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a strict budget burn estimate in status open.
pub fn new_strict_budget_burn_estimate(code: &str, name: &str) -> StrictBudgetBurnEstimate {
    StrictBudgetBurnEstimate {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        expires_at: 0,
        contact_email: "".to_string(),
        ceiling: 0.0,
        archived: false,
        threshold: 0.0,
        country: "".to_string(),
        visible: false,
        floor_value: 0.0,
        status: StrictBudgetBurnEstimateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_budget_burn_estimate("code-130", "name-863");
        assert_eq!(item.status, StrictBudgetBurnEstimateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_strict_budget_burn_estimate("code-130", "name-863");
        if item.move_to(StrictBudgetBurnEstimateStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictBudgetBurnEstimateStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_budget_burn_estimate("code-130", "name-863");
        item.status = StrictBudgetBurnEstimateStatus::Archived;
        assert!(
            item.move_to(StrictBudgetBurnEstimateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
