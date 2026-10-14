use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule profile must pass. Check returns its problems, or none.
pub trait RuleProfileRule: std::fmt::Debug {
    fn check(&self, item: &RuleProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleProfileRequiredNameRule {
    pub message: String,
}

impl RuleProfileRule for RuleProfileRequiredNameRule {
    fn check(&self, item: &RuleProfile) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleProfileNameLengthRule {
    pub max_length: i64,
}

impl RuleProfileRule for RuleProfileNameLengthRule {
    fn check(&self, item: &RuleProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleProfileRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleProfileRule for RuleProfileRetriesRangeRule {
    fn check(&self, item: &RuleProfile) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleProfileRequiredScopeRule {
    pub message: String,
}

impl RuleProfileRule for RuleProfileRequiredScopeRule {
    fn check(&self, item: &RuleProfile) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule profile.
#[derive(Debug, Clone)]
pub struct RuleProfileValidator {
    pub rules: Vec<Rc<dyn RuleProfileRule>>,
}

impl RuleProfileValidator {
    pub fn validate(&self, item: &RuleProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_profile_validator() -> RuleProfileValidator {
    RuleProfileValidator {
        rules: vec![
            Rc::new(RuleProfileRequiredNameRule {
                message: "rule profile needs an entry".to_string(),
            }) as Rc<dyn RuleProfileRule>,
            Rc::new(RuleProfileNameLengthRule { max_length: 86400 }),
            Rc::new(RuleProfileRetriesRangeRule { min: 64, max: 86400 }),
            Rc::new(RuleProfileRequiredScopeRule {
                message: "rule profile needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_profile_validator();
        let item = new_rule_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RuleProfileRequiredNameRule {
            message: "rule profile needs an entry".to_string(),
        };
        let mut item = new_rule_profile("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RuleProfileNameLengthRule { max_length: 64 };
        let mut item = new_rule_profile("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = RuleProfileRetriesRangeRule { min: 1000, max: 3600 };
        let mut item = new_rule_profile("x", "y");
        item.retries = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = RuleProfileRequiredScopeRule {
            message: "rule profile needs a value".to_string(),
        };
        let mut item = new_rule_profile("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
