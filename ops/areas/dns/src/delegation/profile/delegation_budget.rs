use super::*;

use acme_kit::text;

/// Where a delegation budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DelegationBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DelegationBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBudget {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub region: String,
    pub segment: String,
    pub ends_at: i64,
    pub locale: String,
    pub attempts: i64,
    pub channel: String,
    pub status: DelegationBudgetStatus,
}

impl DelegationBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationBudgetStatus) -> bool {
        match self.status {
            DelegationBudgetStatus::Created => {
                match next {
                    DelegationBudgetStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationBudgetStatus::Validated => {
                match next {
                    DelegationBudgetStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationBudgetStatus::Published => {
                return next == DelegationBudgetStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.ends_at as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationBudget {
        DelegationBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            region: self.region.clone(),
            segment: self.segment.clone(),
            ends_at: self.ends_at,
            locale: self.locale.clone(),
            attempts: self.attempts,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a delegation budget in status created.
pub fn new_delegation_budget(code: &str, name: &str) -> DelegationBudget {
    DelegationBudget {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        region: "".to_string(),
        segment: "".to_string(),
        ends_at: 0,
        locale: "".to_string(),
        attempts: 0,
        channel: "".to_string(),
        status: DelegationBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_delegation_budget("code-526", "name-383");
        assert_eq!(item.status, DelegationBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_delegation_budget("code-526", "name-383");
        if item.move_to(DelegationBudgetStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationBudgetStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_delegation_budget("code-526", "name-383");
        item.status = DelegationBudgetStatus::Retired;
        assert!(
            item.move_to(DelegationBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
