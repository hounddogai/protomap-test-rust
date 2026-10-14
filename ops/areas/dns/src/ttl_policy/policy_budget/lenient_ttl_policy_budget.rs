use super::*;

use acme_kit::text;

/// Where a lenient ttl policy budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTtlPolicyBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientTtlPolicyBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientTtlPolicyBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient ttl policy budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyBudget {
    pub code: String,
    pub title: String,
    pub category: String,
    pub margin: f64,
    pub score: f64,
    pub description: String,
    pub status: LenientTtlPolicyBudgetStatus,
}

impl LenientTtlPolicyBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTtlPolicyBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTtlPolicyBudgetStatus) -> bool {
        match self.status {
            LenientTtlPolicyBudgetStatus::Created => {
                match next {
                    LenientTtlPolicyBudgetStatus::Validated | LenientTtlPolicyBudgetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTtlPolicyBudgetStatus::Validated => {
                match next {
                    LenientTtlPolicyBudgetStatus::Published | LenientTtlPolicyBudgetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTtlPolicyBudgetStatus::Published => {
                return next == LenientTtlPolicyBudgetStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientTtlPolicyBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient ttl policy budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient ttl policy budget", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.margin + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientTtlPolicyBudget {
        LenientTtlPolicyBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            margin: self.margin,
            score: self.score,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a lenient ttl policy budget in status created.
pub fn new_lenient_ttl_policy_budget(code: &str, title: &str) -> LenientTtlPolicyBudget {
    LenientTtlPolicyBudget {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        margin: 0.0,
        score: 0.0,
        description: "".to_string(),
        status: LenientTtlPolicyBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_ttl_policy_budget("code-217", "title-847");
        assert_eq!(item.status, LenientTtlPolicyBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_ttl_policy_budget("code-217", "title-847");
        if item.move_to(LenientTtlPolicyBudgetStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientTtlPolicyBudgetStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_ttl_policy_budget("code-217", "title-847");
        item.status = LenientTtlPolicyBudgetStatus::Retired;
        assert!(
            item.move_to(LenientTtlPolicyBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
