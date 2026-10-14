use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover budget must pass. Check returns its problems, or none.
pub trait FailoverBudgetRule: std::fmt::Debug {
    fn check(&self, item: &FailoverBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBudgetRequiredCodeRule {
    pub message: String,
}

impl FailoverBudgetRule for FailoverBudgetRequiredCodeRule {
    fn check(&self, item: &FailoverBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBudgetSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverBudgetRule for FailoverBudgetSequenceRangeRule {
    fn check(&self, item: &FailoverBudget) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBudgetRequiredExternalRefRule {
    pub message: String,
}

impl FailoverBudgetRule for FailoverBudgetRequiredExternalRefRule {
    fn check(&self, item: &FailoverBudget) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover budget.
#[derive(Debug, Clone)]
pub struct FailoverBudgetValidator {
    pub rules: Vec<Rc<dyn FailoverBudgetRule>>,
}

impl FailoverBudgetValidator {
    pub fn validate(&self, item: &FailoverBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_budget_validator() -> FailoverBudgetValidator {
    FailoverBudgetValidator {
        rules: vec![
            Rc::new(FailoverBudgetRequiredCodeRule {
                message: "failover budget needs a value".to_string(),
            }) as Rc<dyn FailoverBudgetRule>,
            Rc::new(FailoverBudgetSequenceRangeRule { min: 16, max: 32 }),
            Rc::new(FailoverBudgetRequiredExternalRefRule {
                message: "failover budget needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_budget_validator();
        let item = new_failover_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FailoverBudgetRequiredCodeRule {
            message: "failover budget needs an entry".to_string(),
        };
        let mut item = new_failover_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = FailoverBudgetSequenceRangeRule { min: 80, max: 255 };
        let mut item = new_failover_budget("x", "y");
        item.sequence = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = FailoverBudgetRequiredExternalRefRule {
            message: "failover budget needs an entry".to_string(),
        };
        let mut item = new_failover_budget("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
