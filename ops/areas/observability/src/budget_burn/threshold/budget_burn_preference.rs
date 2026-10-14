use super::*;

use acme_kit::text;

/// Where a budget burn preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnPreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BudgetBurnPreferenceStatus {
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

impl std::fmt::Display for BudgetBurnPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn preference as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPreference {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub key_id: String,
    pub issuer: String,
    pub description: String,
    pub namespace: String,
    pub score: f64,
    pub status: BudgetBurnPreferenceStatus,
}

impl BudgetBurnPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnPreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnPreferenceStatus) -> bool {
        match self.status {
            BudgetBurnPreferenceStatus::Draft => {
                match next {
                    BudgetBurnPreferenceStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnPreferenceStatus::Pending => {
                match next {
                    BudgetBurnPreferenceStatus::Active | BudgetBurnPreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnPreferenceStatus::Active => {
                match next {
                    BudgetBurnPreferenceStatus::Suspended | BudgetBurnPreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnPreferenceStatus::Suspended => {
                return next == BudgetBurnPreferenceStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> BudgetBurnPreference {
        BudgetBurnPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            key_id: self.key_id.clone(),
            issuer: self.issuer.clone(),
            description: self.description.clone(),
            namespace: self.namespace.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a budget burn preference in status draft.
pub fn new_budget_burn_preference(code: &str, title: &str) -> BudgetBurnPreference {
    BudgetBurnPreference {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        key_id: "".to_string(),
        issuer: "".to_string(),
        description: "".to_string(),
        namespace: "".to_string(),
        score: 0.0,
        status: BudgetBurnPreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_budget_burn_preference("code-104", "title-405");
        assert_eq!(item.status, BudgetBurnPreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_budget_burn_preference("code-104", "title-405");
        if item.move_to(BudgetBurnPreferenceStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnPreferenceStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_budget_burn_preference("code-104", "title-405");
        item.status = BudgetBurnPreferenceStatus::Closed;
        assert!(
            item.move_to(BudgetBurnPreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
