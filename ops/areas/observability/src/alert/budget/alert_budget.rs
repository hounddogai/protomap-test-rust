use super::*;

use acme_kit::text;

/// Where a alert budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertBudgetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AlertBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AlertBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One alert budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertBudget {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub source: String,
    pub contact_email: String,
    pub category: String,
    pub revision: i64,
    pub amount: f64,
    pub status: AlertBudgetStatus,
}

impl AlertBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertBudgetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertBudgetStatus) -> bool {
        match self.status {
            AlertBudgetStatus::Proposed => {
                match next {
                    AlertBudgetStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertBudgetStatus::Approved => {
                return next == AlertBudgetStatus::Rejected || next == AlertBudgetStatus::Withdrawn;
            }
            AlertBudgetStatus::Rejected => {
                match next {
                    AlertBudgetStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AlertBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "alert budget", self.code)
    }

    pub fn net_value(&self) -> f64 {
        (self.score * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertBudget {
        AlertBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            source: self.source.clone(),
            contact_email: self.contact_email.clone(),
            category: self.category.clone(),
            revision: self.revision,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a alert budget in status proposed.
pub fn new_alert_budget(code: &str, name: &str) -> AlertBudget {
    AlertBudget {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        source: "".to_string(),
        contact_email: "".to_string(),
        category: "".to_string(),
        revision: 0,
        amount: 0.0,
        status: AlertBudgetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_alert_budget("code-867", "name-901");
        assert_eq!(item.status, AlertBudgetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_alert_budget("code-867", "name-901");
        if item.move_to(AlertBudgetStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertBudgetStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_alert_budget("code-867", "name-901");
        item.status = AlertBudgetStatus::Withdrawn;
        assert!(
            item.move_to(AlertBudgetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
