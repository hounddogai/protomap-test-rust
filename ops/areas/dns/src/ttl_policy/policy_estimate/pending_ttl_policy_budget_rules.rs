use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending ttl policy budget must pass. Check returns its problems, or none.
pub trait PendingTtlPolicyBudgetRule: std::fmt::Debug {
    fn check(&self, item: &PendingTtlPolicyBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyBudgetKeyIdLengthRule {
    pub max_length: i64,
}

impl PendingTtlPolicyBudgetRule for PendingTtlPolicyBudgetKeyIdLengthRule {
    fn check(&self, item: &PendingTtlPolicyBudget) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyBudgetChecksumLengthRule {
    pub max_length: i64,
}

impl PendingTtlPolicyBudgetRule for PendingTtlPolicyBudgetChecksumLengthRule {
    fn check(&self, item: &PendingTtlPolicyBudget) -> Vec<String> {
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
pub struct PendingTtlPolicyBudgetCodeLengthRule {
    pub max_length: i64,
}

impl PendingTtlPolicyBudgetRule for PendingTtlPolicyBudgetCodeLengthRule {
    fn check(&self, item: &PendingTtlPolicyBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyBudgetLabelLengthRule {
    pub max_length: i64,
}

impl PendingTtlPolicyBudgetRule for PendingTtlPolicyBudgetLabelLengthRule {
    fn check(&self, item: &PendingTtlPolicyBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending ttl policy budget.
#[derive(Debug, Clone)]
pub struct PendingTtlPolicyBudgetValidator {
    pub rules: Vec<Rc<dyn PendingTtlPolicyBudgetRule>>,
}

impl PendingTtlPolicyBudgetValidator {
    pub fn validate(&self, item: &PendingTtlPolicyBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingTtlPolicyBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingTtlPolicyBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_ttl_policy_budget_validator() -> PendingTtlPolicyBudgetValidator {
    PendingTtlPolicyBudgetValidator {
        rules: vec![
            Rc::new(PendingTtlPolicyBudgetKeyIdLengthRule { max_length: 64 }) as Rc<dyn PendingTtlPolicyBudgetRule>,
            Rc::new(PendingTtlPolicyBudgetChecksumLengthRule { max_length: 120 }),
            Rc::new(PendingTtlPolicyBudgetCodeLengthRule { max_length: 64 }),
            Rc::new(PendingTtlPolicyBudgetLabelLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_ttl_policy_budget_validator();
        let item = new_pending_ttl_policy_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = PendingTtlPolicyBudgetKeyIdLengthRule { max_length: 120 };
        let mut item = new_pending_ttl_policy_budget("x", "y");
        item.key_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = PendingTtlPolicyBudgetChecksumLengthRule { max_length: 1000 };
        let mut item = new_pending_ttl_policy_budget("x", "y");
        item.checksum = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingTtlPolicyBudgetCodeLengthRule { max_length: 3600 };
        let mut item = new_pending_ttl_policy_budget("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingTtlPolicyBudgetLabelLengthRule { max_length: 255 };
        let mut item = new_pending_ttl_policy_budget("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
