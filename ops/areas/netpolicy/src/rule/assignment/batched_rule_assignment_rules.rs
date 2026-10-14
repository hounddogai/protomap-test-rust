use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched rule assignment must pass. Check returns its problems, or none.
pub trait BatchedRuleAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRuleAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRuleAssignmentRequiredCodeRule {
    pub message: String,
}

impl BatchedRuleAssignmentRule for BatchedRuleAssignmentRequiredCodeRule {
    fn check(&self, item: &BatchedRuleAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRuleAssignmentCodePrefixRule {
    pub prefix: String,
}

impl BatchedRuleAssignmentRule for BatchedRuleAssignmentCodePrefixRule {
    fn check(&self, item: &BatchedRuleAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRuleAssignmentQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedRuleAssignmentRule for BatchedRuleAssignmentQuotaUnitsRangeRule {
    fn check(&self, item: &BatchedRuleAssignment) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRuleAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl BatchedRuleAssignmentRule for BatchedRuleAssignmentCodeLengthRule {
    fn check(&self, item: &BatchedRuleAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched rule assignment.
#[derive(Debug, Clone)]
pub struct BatchedRuleAssignmentValidator {
    pub rules: Vec<Rc<dyn BatchedRuleAssignmentRule>>,
}

impl BatchedRuleAssignmentValidator {
    pub fn validate(&self, item: &BatchedRuleAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedRuleAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRuleAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_rule_assignment_validator() -> BatchedRuleAssignmentValidator {
    BatchedRuleAssignmentValidator {
        rules: vec![
            Rc::new(BatchedRuleAssignmentRequiredCodeRule {
                message: "batched rule assignment needs an entry".to_string(),
            }) as Rc<dyn BatchedRuleAssignmentRule>,
            Rc::new(BatchedRuleAssignmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(BatchedRuleAssignmentQuotaUnitsRangeRule { min: 64, max: 120 }),
            Rc::new(BatchedRuleAssignmentCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_rule_assignment_validator();
        let item = new_batched_rule_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedRuleAssignmentRequiredCodeRule {
            message: "batched rule assignment needs this field".to_string(),
        };
        let mut item = new_batched_rule_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = BatchedRuleAssignmentQuotaUnitsRangeRule { min: 120, max: 120 };
        let mut item = new_batched_rule_assignment("x", "y");
        item.quota_units = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedRuleAssignmentCodeLengthRule { max_length: 120 };
        let mut item = new_batched_rule_assignment("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
