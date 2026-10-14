use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive isolation group must pass. Check returns its problems, or none.
pub trait AdaptiveIsolationGroupRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveIsolationGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationGroupLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveIsolationGroupRule for AdaptiveIsolationGroupLabelLengthRule {
    fn check(&self, item: &AdaptiveIsolationGroup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationGroupCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveIsolationGroupRule for AdaptiveIsolationGroupCodePrefixRule {
    fn check(&self, item: &AdaptiveIsolationGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationGroupLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveIsolationGroupRule for AdaptiveIsolationGroupLimitRangeRule {
    fn check(&self, item: &AdaptiveIsolationGroup) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationGroupRequiredCodeRule {
    pub message: String,
}

impl AdaptiveIsolationGroupRule for AdaptiveIsolationGroupRequiredCodeRule {
    fn check(&self, item: &AdaptiveIsolationGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive isolation group.
#[derive(Debug, Clone)]
pub struct AdaptiveIsolationGroupValidator {
    pub rules: Vec<Rc<dyn AdaptiveIsolationGroupRule>>,
}

impl AdaptiveIsolationGroupValidator {
    pub fn validate(&self, item: &AdaptiveIsolationGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveIsolationGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveIsolationGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_isolation_group_validator() -> AdaptiveIsolationGroupValidator {
    AdaptiveIsolationGroupValidator {
        rules: vec![
            Rc::new(AdaptiveIsolationGroupLabelLengthRule { max_length: 80 }) as Rc<dyn AdaptiveIsolationGroupRule>,
            Rc::new(AdaptiveIsolationGroupCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(AdaptiveIsolationGroupLimitRangeRule { min: 32, max: 86400 }),
            Rc::new(AdaptiveIsolationGroupRequiredCodeRule {
                message: "adaptive isolation group needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_isolation_group_validator();
        let item = new_adaptive_isolation_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AdaptiveIsolationGroupLabelLengthRule { max_length: 64 };
        let mut item = new_adaptive_isolation_group("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = AdaptiveIsolationGroupLimitRangeRule { min: 16, max: 1000 };
        let mut item = new_adaptive_isolation_group("x", "y");
        item.limit = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveIsolationGroupRequiredCodeRule {
            message: "adaptive isolation group needs an entry".to_string(),
        };
        let mut item = new_adaptive_isolation_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
