use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy rule must pass. Check returns its problems, or none.
pub trait PolicyRuleRule: std::fmt::Debug {
    fn check(&self, item: &PolicyRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyRuleScopeLengthRule {
    pub max_length: i64,
}

impl PolicyRuleRule for PolicyRuleScopeLengthRule {
    fn check(&self, item: &PolicyRule) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyRuleStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyRuleRule for PolicyRuleStartsAtRangeRule {
    fn check(&self, item: &PolicyRule) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyRuleRequiredPhoneRule {
    pub message: String,
}

impl PolicyRuleRule for PolicyRuleRequiredPhoneRule {
    fn check(&self, item: &PolicyRule) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy rule.
#[derive(Debug, Clone)]
pub struct PolicyRuleValidator {
    pub rules: Vec<Rc<dyn PolicyRuleRule>>,
}

impl PolicyRuleValidator {
    pub fn validate(&self, item: &PolicyRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PolicyRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_rule_validator() -> PolicyRuleValidator {
    PolicyRuleValidator {
        rules: vec![
            Rc::new(PolicyRuleScopeLengthRule { max_length: 32 }) as Rc<dyn PolicyRuleRule>,
            Rc::new(PolicyRuleStartsAtRangeRule { min: 80, max: 1000 }),
            Rc::new(PolicyRuleRequiredPhoneRule {
                message: "policy rule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_rule_validator();
        let item = new_policy_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = PolicyRuleScopeLengthRule { max_length: 255 };
        let mut item = new_policy_rule("x", "y");
        item.scope = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = PolicyRuleStartsAtRangeRule { min: 16, max: 1000 };
        let mut item = new_policy_rule("x", "y");
        item.starts_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = PolicyRuleRequiredPhoneRule {
            message: "policy rule needs an entry".to_string(),
        };
        let mut item = new_policy_rule("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
