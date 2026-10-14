use super::*;

use acme_kit::text;

/// Where a weighted approval budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedApprovalBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedApprovalBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedApprovalBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted approval budget as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedApprovalBudget {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub name: String,
    pub tenant_id: String,
    pub notes: String,
    pub budget: f64,
    pub source: String,
    pub starts_at: i64,
    pub scope: String,
    pub status: WeightedApprovalBudgetStatus,
}

impl WeightedApprovalBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedApprovalBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedApprovalBudgetStatus) -> bool {
        match self.status {
            WeightedApprovalBudgetStatus::Created => {
                match next {
                    WeightedApprovalBudgetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedApprovalBudgetStatus::Validated => {
                return next == WeightedApprovalBudgetStatus::Retired;
            }
            WeightedApprovalBudgetStatus::Published => {
                match next {
                    WeightedApprovalBudgetStatus::Retired => {
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

    pub fn move_to(&mut self, next: WeightedApprovalBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted approval budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.rate - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedApprovalBudget {
        WeightedApprovalBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            notes: self.notes.clone(),
            budget: self.budget,
            source: self.source.clone(),
            starts_at: self.starts_at,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a weighted approval budget in status created.
pub fn new_weighted_approval_budget(code: &str, title: &str) -> WeightedApprovalBudget {
    WeightedApprovalBudget {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        name: "".to_string(),
        tenant_id: "".to_string(),
        notes: "".to_string(),
        budget: 0.0,
        source: "".to_string(),
        starts_at: 0,
        scope: "".to_string(),
        status: WeightedApprovalBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_approval_budget("code-783", "title-897");
        assert_eq!(item.status, WeightedApprovalBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_weighted_approval_budget("code-783", "title-897");
        if item.move_to(WeightedApprovalBudgetStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedApprovalBudgetStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_approval_budget("code-783", "title-897");
        item.status = WeightedApprovalBudgetStatus::Retired;
        assert!(
            item.move_to(WeightedApprovalBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
