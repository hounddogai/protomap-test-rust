use super::*;

use acme_kit::text;

/// Where a scheduled nonce budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledNonceBudgetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledNonceBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledNonceBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled nonce budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceBudget {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub issuer: String,
    pub threshold: f64,
    pub currency: String,
    pub key_id: String,
    pub status: ScheduledNonceBudgetStatus,
}

impl ScheduledNonceBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledNonceBudgetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledNonceBudgetStatus) -> bool {
        match self.status {
            ScheduledNonceBudgetStatus::Proposed => {
                match next {
                    ScheduledNonceBudgetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledNonceBudgetStatus::Approved => {
                match next {
                    ScheduledNonceBudgetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledNonceBudgetStatus::Rejected => {
                match next {
                    ScheduledNonceBudgetStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ScheduledNonceBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled nonce budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled nonce budget", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.threshold - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledNonceBudget {
        ScheduledNonceBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            issuer: self.issuer.clone(),
            threshold: self.threshold,
            currency: self.currency.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled nonce budget in status proposed.
pub fn new_scheduled_nonce_budget(code: &str, name: &str) -> ScheduledNonceBudget {
    ScheduledNonceBudget {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        issuer: "".to_string(),
        threshold: 0.0,
        currency: "".to_string(),
        key_id: "".to_string(),
        status: ScheduledNonceBudgetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_nonce_budget("code-780", "name-3");
        assert_eq!(item.status, ScheduledNonceBudgetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_scheduled_nonce_budget("code-780", "name-3");
        if item.move_to(ScheduledNonceBudgetStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledNonceBudgetStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_nonce_budget("code-780", "name-3");
        item.status = ScheduledNonceBudgetStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledNonceBudgetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
