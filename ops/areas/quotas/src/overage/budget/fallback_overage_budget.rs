use super::*;

use acme_kit::text;

/// Where a fallback overage budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackOverageBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackOverageBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for FallbackOverageBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback overage budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageBudget {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub host: String,
    pub floor_value: f64,
    pub status: FallbackOverageBudgetStatus,
}

impl FallbackOverageBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackOverageBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackOverageBudgetStatus) -> bool {
        match self.status {
            FallbackOverageBudgetStatus::Draft => {
                match next {
                    FallbackOverageBudgetStatus::Suspended | FallbackOverageBudgetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackOverageBudgetStatus::Pending => {
                return next == FallbackOverageBudgetStatus::Active || next == FallbackOverageBudgetStatus::Closed;
            }
            FallbackOverageBudgetStatus::Active => {
                return next == FallbackOverageBudgetStatus::Suspended;
            }
            FallbackOverageBudgetStatus::Suspended => {
                match next {
                    FallbackOverageBudgetStatus::Closed => {
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

    pub fn move_to(&mut self, next: FallbackOverageBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback overage budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.floor_value - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackOverageBudget {
        FallbackOverageBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            host: self.host.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback overage budget in status draft.
pub fn new_fallback_overage_budget(code: &str, label: &str) -> FallbackOverageBudget {
    FallbackOverageBudget {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        host: "".to_string(),
        floor_value: 0.0,
        status: FallbackOverageBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_overage_budget("code-907", "label-654");
        assert_eq!(item.status, FallbackOverageBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_fallback_overage_budget("code-907", "label-654");
        if item.move_to(FallbackOverageBudgetStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackOverageBudgetStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_overage_budget("code-907", "label-654");
        item.status = FallbackOverageBudgetStatus::Closed;
        assert!(
            item.move_to(FallbackOverageBudgetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
