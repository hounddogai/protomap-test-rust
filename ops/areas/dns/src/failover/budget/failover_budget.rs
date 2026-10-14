use super::*;

use acme_kit::text;

/// Where a failover budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverBudgetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FailoverBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FailoverBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover budget as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBudget {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub contact_email: String,
    pub sequence: i64,
    pub status: FailoverBudgetStatus,
}

impl FailoverBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverBudgetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverBudgetStatus) -> bool {
        match self.status {
            FailoverBudgetStatus::Requested => {
                return next == FailoverBudgetStatus::Granted || next == FailoverBudgetStatus::Revoked;
            }
            FailoverBudgetStatus::Granted => {
                match next {
                    FailoverBudgetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverBudgetStatus::Revoked => {
                return next == FailoverBudgetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> FailoverBudget {
        FailoverBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            contact_email: self.contact_email.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a failover budget in status requested.
pub fn new_failover_budget(code: &str, title: &str) -> FailoverBudget {
    FailoverBudget {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        contact_email: "".to_string(),
        sequence: 0,
        status: FailoverBudgetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_failover_budget("code-649", "title-689");
        assert_eq!(item.status, FailoverBudgetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_failover_budget("code-649", "title-689");
        if item.move_to(FailoverBudgetStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverBudgetStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_failover_budget("code-649", "title-689");
        item.status = FailoverBudgetStatus::Expired;
        assert!(
            item.move_to(FailoverBudgetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
