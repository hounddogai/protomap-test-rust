use super::*;

use acme_kit::text;

/// Where a default flow log budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultFlowLogBudgetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultFlowLogBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultFlowLogBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default flow log budget as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogBudget {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub rate: f64,
    pub created_at: i64,
    pub batch_size: i64,
    pub phone: String,
    pub status: DefaultFlowLogBudgetStatus,
}

impl DefaultFlowLogBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultFlowLogBudgetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultFlowLogBudgetStatus) -> bool {
        match self.status {
            DefaultFlowLogBudgetStatus::Requested => {
                match next {
                    DefaultFlowLogBudgetStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFlowLogBudgetStatus::Granted => {
                match next {
                    DefaultFlowLogBudgetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFlowLogBudgetStatus::Revoked => {
                return next == DefaultFlowLogBudgetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultFlowLogBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default flow log budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.rate - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultFlowLogBudget {
        DefaultFlowLogBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            rate: self.rate,
            created_at: self.created_at,
            batch_size: self.batch_size,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default flow log budget in status requested.
pub fn new_default_flow_log_budget(code: &str, label: &str) -> DefaultFlowLogBudget {
    DefaultFlowLogBudget {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        rate: 0.0,
        created_at: 0,
        batch_size: 0,
        phone: "".to_string(),
        status: DefaultFlowLogBudgetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_flow_log_budget("code-710", "label-971");
        assert_eq!(item.status, DefaultFlowLogBudgetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_default_flow_log_budget("code-710", "label-971");
        if item.move_to(DefaultFlowLogBudgetStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultFlowLogBudgetStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_flow_log_budget("code-710", "label-971");
        item.status = DefaultFlowLogBudgetStatus::Expired;
        assert!(
            item.move_to(DefaultFlowLogBudgetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
