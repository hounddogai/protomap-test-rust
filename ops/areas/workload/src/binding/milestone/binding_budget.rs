use super::*;

use acme_kit::text;

/// Where a binding budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingBudgetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BindingBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BindingBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding budget as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingBudget {
    pub code: String,
    pub title: String,
    pub label: String,
    pub updated_at: i64,
    pub reviewer: String,
    pub sequence: i64,
    pub scope: String,
    pub principal: String,
    pub checksum: String,
    pub status: BindingBudgetStatus,
}

impl BindingBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingBudgetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingBudgetStatus) -> bool {
        match self.status {
            BindingBudgetStatus::Proposed => {
                return next == BindingBudgetStatus::Rejected || next == BindingBudgetStatus::Withdrawn;
            }
            BindingBudgetStatus::Approved => {
                match next {
                    BindingBudgetStatus::Rejected | BindingBudgetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingBudgetStatus::Rejected => {
                return next == BindingBudgetStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.updated_at as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingBudget {
        BindingBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            reviewer: self.reviewer.clone(),
            sequence: self.sequence,
            scope: self.scope.clone(),
            principal: self.principal.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a binding budget in status proposed.
pub fn new_binding_budget(code: &str, title: &str) -> BindingBudget {
    BindingBudget {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        updated_at: 0,
        reviewer: "".to_string(),
        sequence: 0,
        scope: "".to_string(),
        principal: "".to_string(),
        checksum: "".to_string(),
        status: BindingBudgetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_binding_budget("code-499", "title-185");
        assert_eq!(item.status, BindingBudgetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_binding_budget("code-499", "title-185");
        if item.move_to(BindingBudgetStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingBudgetStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_binding_budget("code-499", "title-185");
        item.status = BindingBudgetStatus::Withdrawn;
        assert!(
            item.move_to(BindingBudgetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
