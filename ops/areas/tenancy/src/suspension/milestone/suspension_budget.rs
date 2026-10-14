use super::*;

use acme_kit::text;

/// Where a suspension budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SuspensionBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SuspensionBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension budget as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBudget {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub locale: String,
    pub description: String,
    pub score: f64,
    pub status: SuspensionBudgetStatus,
}

impl SuspensionBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionBudgetStatus) -> bool {
        match self.status {
            SuspensionBudgetStatus::Created => {
                match next {
                    SuspensionBudgetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionBudgetStatus::Validated => {
                return next == SuspensionBudgetStatus::Retired;
            }
            SuspensionBudgetStatus::Published => {
                match next {
                    SuspensionBudgetStatus::Retired => {
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

    pub fn move_to(&mut self, next: SuspensionBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> SuspensionBudget {
        SuspensionBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            locale: self.locale.clone(),
            description: self.description.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a suspension budget in status created.
pub fn new_suspension_budget(code: &str, label: &str) -> SuspensionBudget {
    SuspensionBudget {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        locale: "".to_string(),
        description: "".to_string(),
        score: 0.0,
        status: SuspensionBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_suspension_budget("code-616", "label-601");
        assert_eq!(item.status, SuspensionBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_suspension_budget("code-616", "label-601");
        if item.move_to(SuspensionBudgetStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionBudgetStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_suspension_budget("code-616", "label-601");
        item.status = SuspensionBudgetStatus::Retired;
        assert!(
            item.move_to(SuspensionBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
