use super::*;

use acme_kit::text;

/// Where a ttl policy budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyBudgetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TtlPolicyBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for TtlPolicyBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ttl policy budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBudget {
    pub code: String,
    pub title: String,
    pub region: String,
    pub ratio: f64,
    pub namespace: String,
    pub cluster: String,
    pub currency: String,
    pub checksum: String,
    pub principal: String,
    pub status: TtlPolicyBudgetStatus,
}

impl TtlPolicyBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyBudgetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyBudgetStatus) -> bool {
        match self.status {
            TtlPolicyBudgetStatus::New => {
                return next == TtlPolicyBudgetStatus::Running || next == TtlPolicyBudgetStatus::Done;
            }
            TtlPolicyBudgetStatus::Queued => {
                return next == TtlPolicyBudgetStatus::Running;
            }
            TtlPolicyBudgetStatus::Running => {
                return next == TtlPolicyBudgetStatus::Done || next == TtlPolicyBudgetStatus::Failed;
            }
            TtlPolicyBudgetStatus::Done => {
                return next == TtlPolicyBudgetStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy budget {} cannot move from {} to {}",
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
        self.ratio
    }

    pub fn copy(&self) -> TtlPolicyBudget {
        TtlPolicyBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            ratio: self.ratio,
            namespace: self.namespace.clone(),
            cluster: self.cluster.clone(),
            currency: self.currency.clone(),
            checksum: self.checksum.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a ttl policy budget in status new.
pub fn new_ttl_policy_budget(code: &str, title: &str) -> TtlPolicyBudget {
    TtlPolicyBudget {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        ratio: 0.0,
        namespace: "".to_string(),
        cluster: "".to_string(),
        currency: "".to_string(),
        checksum: "".to_string(),
        principal: "".to_string(),
        status: TtlPolicyBudgetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_ttl_policy_budget("code-435", "title-419");
        assert_eq!(item.status, TtlPolicyBudgetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_ttl_policy_budget("code-435", "title-419");
        if item.move_to(TtlPolicyBudgetStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyBudgetStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_ttl_policy_budget("code-435", "title-419");
        item.status = TtlPolicyBudgetStatus::Failed;
        assert!(
            item.move_to(TtlPolicyBudgetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
