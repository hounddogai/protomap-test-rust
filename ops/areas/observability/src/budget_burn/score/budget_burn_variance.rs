use super::*;

use acme_kit::text;

/// Where a budget burn variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BudgetBurnVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for BudgetBurnVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One budget burn variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnVariance {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub version: i64,
    pub duration_minutes: i64,
    pub unit: String,
    pub approved: bool,
    pub timezone: String,
    pub status: BudgetBurnVarianceStatus,
}

impl BudgetBurnVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnVarianceStatus) -> bool {
        match self.status {
            BudgetBurnVarianceStatus::New => {
                return next == BudgetBurnVarianceStatus::Running || next == BudgetBurnVarianceStatus::Failed;
            }
            BudgetBurnVarianceStatus::Queued => {
                return next == BudgetBurnVarianceStatus::Done;
            }
            BudgetBurnVarianceStatus::Running => {
                match next {
                    BudgetBurnVarianceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnVarianceStatus::Done => {
                return next == BudgetBurnVarianceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnVariance {
        BudgetBurnVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            version: self.version,
            duration_minutes: self.duration_minutes,
            unit: self.unit.clone(),
            approved: self.approved,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a budget burn variance in status new.
pub fn new_budget_burn_variance(code: &str, label: &str) -> BudgetBurnVariance {
    BudgetBurnVariance {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        version: 0,
        duration_minutes: 0,
        unit: "".to_string(),
        approved: false,
        timezone: "".to_string(),
        status: BudgetBurnVarianceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_budget_burn_variance("code-694", "label-189");
        assert_eq!(item.status, BudgetBurnVarianceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_budget_burn_variance("code-694", "label-189");
        if item.move_to(BudgetBurnVarianceStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnVarianceStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_budget_burn_variance("code-694", "label-189");
        item.status = BudgetBurnVarianceStatus::Failed;
        assert!(
            item.move_to(BudgetBurnVarianceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
