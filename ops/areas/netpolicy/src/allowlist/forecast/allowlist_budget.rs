use super::*;

use acme_kit::text;

/// Where a allowlist budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistBudgetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AllowlistBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AllowlistBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBudget {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub description: String,
    pub limit: i64,
    pub grace_days: i64,
    pub threshold: f64,
    pub external_ref: String,
    pub reviewer: String,
    pub owner_id: String,
    pub cost: f64,
    pub status: AllowlistBudgetStatus,
}

impl AllowlistBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistBudgetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistBudgetStatus) -> bool {
        match self.status {
            AllowlistBudgetStatus::Requested => {
                match next {
                    AllowlistBudgetStatus::Granted | AllowlistBudgetStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistBudgetStatus::Granted => {
                match next {
                    AllowlistBudgetStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistBudgetStatus::Revoked => {
                return next == AllowlistBudgetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.cost * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistBudget {
        AllowlistBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            description: self.description.clone(),
            limit: self.limit,
            grace_days: self.grace_days,
            threshold: self.threshold,
            external_ref: self.external_ref.clone(),
            reviewer: self.reviewer.clone(),
            owner_id: self.owner_id.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a allowlist budget in status requested.
pub fn new_allowlist_budget(code: &str, name: &str) -> AllowlistBudget {
    AllowlistBudget {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        description: "".to_string(),
        limit: 0,
        grace_days: 0,
        threshold: 0.0,
        external_ref: "".to_string(),
        reviewer: "".to_string(),
        owner_id: "".to_string(),
        cost: 0.0,
        status: AllowlistBudgetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_allowlist_budget("code-856", "name-319");
        assert_eq!(item.status, AllowlistBudgetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_allowlist_budget("code-856", "name-319");
        if item.move_to(AllowlistBudgetStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistBudgetStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_allowlist_budget("code-856", "name-319");
        item.status = AllowlistBudgetStatus::Expired;
        assert!(
            item.move_to(AllowlistBudgetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
