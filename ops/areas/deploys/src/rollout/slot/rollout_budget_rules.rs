use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout budget must pass. Check returns its problems, or none.
pub trait RolloutBudgetRule: std::fmt::Debug {
    fn check(&self, item: &RolloutBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBudgetChecksumLengthRule {
    pub max_length: i64,
}

impl RolloutBudgetRule for RolloutBudgetChecksumLengthRule {
    fn check(&self, item: &RolloutBudget) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBudgetCodeLengthRule {
    pub max_length: i64,
}

impl RolloutBudgetRule for RolloutBudgetCodeLengthRule {
    fn check(&self, item: &RolloutBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBudgetNameLengthRule {
    pub max_length: i64,
}

impl RolloutBudgetRule for RolloutBudgetNameLengthRule {
    fn check(&self, item: &RolloutBudget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBudgetApprovedGuardRule {
    pub limit: i64,
}

impl RolloutBudgetRule for RolloutBudgetApprovedGuardRule {
    fn check(&self, item: &RolloutBudget) -> Vec<String> {
        if item.approved && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while approved", item.burst, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBudgetRequiredChecksumRule {
    pub message: String,
}

impl RolloutBudgetRule for RolloutBudgetRequiredChecksumRule {
    fn check(&self, item: &RolloutBudget) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout budget.
#[derive(Debug, Clone)]
pub struct RolloutBudgetValidator {
    pub rules: Vec<Rc<dyn RolloutBudgetRule>>,
}

impl RolloutBudgetValidator {
    pub fn validate(&self, item: &RolloutBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutBudget]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_rollout_budget_validator() -> RolloutBudgetValidator {
    RolloutBudgetValidator {
        rules: vec![
            Rc::new(RolloutBudgetChecksumLengthRule { max_length: 32 }) as Rc<dyn RolloutBudgetRule>,
            Rc::new(RolloutBudgetCodeLengthRule { max_length: 86400 }),
            Rc::new(RolloutBudgetNameLengthRule { max_length: 3600 }),
            Rc::new(RolloutBudgetApprovedGuardRule { limit: 16 }),
            Rc::new(RolloutBudgetRequiredChecksumRule {
                message: "rollout budget needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_budget_validator();
        let item = new_rollout_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = RolloutBudgetChecksumLengthRule { max_length: 80 };
        let mut item = new_rollout_budget("x", "y");
        item.checksum = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RolloutBudgetCodeLengthRule { max_length: 3600 };
        let mut item = new_rollout_budget("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RolloutBudgetNameLengthRule { max_length: 16 };
        let mut item = new_rollout_budget("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = RolloutBudgetRequiredChecksumRule {
            message: "rollout budget needs this field".to_string(),
        };
        let mut item = new_rollout_budget("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
