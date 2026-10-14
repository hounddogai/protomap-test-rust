use super::*;

use acme_kit::text;

/// Where a weighted retention budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRetentionBudgetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl WeightedRetentionBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for WeightedRetentionBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted retention budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionBudget {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub amount: f64,
    pub principal: String,
    pub locale: String,
    pub checksum: String,
    pub contact_email: String,
    pub audience: String,
    pub burst: i64,
    pub status: WeightedRetentionBudgetStatus,
}

impl WeightedRetentionBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRetentionBudgetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRetentionBudgetStatus) -> bool {
        match self.status {
            WeightedRetentionBudgetStatus::Requested => {
                match next {
                    WeightedRetentionBudgetStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRetentionBudgetStatus::Granted => {
                match next {
                    WeightedRetentionBudgetStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRetentionBudgetStatus::Revoked => {
                return next == WeightedRetentionBudgetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedRetentionBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted retention budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.burst as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRetentionBudget {
        WeightedRetentionBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            amount: self.amount,
            principal: self.principal.clone(),
            locale: self.locale.clone(),
            checksum: self.checksum.clone(),
            contact_email: self.contact_email.clone(),
            audience: self.audience.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a weighted retention budget in status requested.
pub fn new_weighted_retention_budget(code: &str, name: &str) -> WeightedRetentionBudget {
    WeightedRetentionBudget {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        amount: 0.0,
        principal: "".to_string(),
        locale: "".to_string(),
        checksum: "".to_string(),
        contact_email: "".to_string(),
        audience: "".to_string(),
        burst: 0,
        status: WeightedRetentionBudgetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_weighted_retention_budget("code-928", "name-220");
        assert_eq!(item.status, WeightedRetentionBudgetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_weighted_retention_budget("code-928", "name-220");
        if item.move_to(WeightedRetentionBudgetStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedRetentionBudgetStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_weighted_retention_budget("code-928", "name-220");
        item.status = WeightedRetentionBudgetStatus::Expired;
        assert!(
            item.move_to(WeightedRetentionBudgetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
