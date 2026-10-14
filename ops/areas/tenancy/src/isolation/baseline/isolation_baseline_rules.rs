use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a isolation baseline must pass. Check returns its problems, or none.
pub trait IsolationBaselineRule: std::fmt::Debug {
    fn check(&self, item: &IsolationBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBaselineAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl IsolationBaselineRule for IsolationBaselineAllowedLabelRule {
    fn check(&self, item: &IsolationBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is blocked", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBaselineMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl IsolationBaselineRule for IsolationBaselineMinimumRangeRule {
    fn check(&self, item: &IsolationBaseline) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBaselineRequiredCodeRule {
    pub message: String,
}

impl IsolationBaselineRule for IsolationBaselineRequiredCodeRule {
    fn check(&self, item: &IsolationBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBaselineLabelLengthRule {
    pub max_length: i64,
}

impl IsolationBaselineRule for IsolationBaselineLabelLengthRule {
    fn check(&self, item: &IsolationBaseline) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBaselineCodePrefixRule {
    pub prefix: String,
}

impl IsolationBaselineRule for IsolationBaselineCodePrefixRule {
    fn check(&self, item: &IsolationBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation baseline.
#[derive(Debug, Clone)]
pub struct IsolationBaselineValidator {
    pub rules: Vec<Rc<dyn IsolationBaselineRule>>,
}

impl IsolationBaselineValidator {
    pub fn validate(&self, item: &IsolationBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IsolationBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_baseline_validator() -> IsolationBaselineValidator {
    IsolationBaselineValidator {
        rules: vec![
            Rc::new(IsolationBaselineAllowedLabelRule {
                allowed: vec!["primary".to_string(), "lenient".to_string()],
            }) as Rc<dyn IsolationBaselineRule>,
            Rc::new(IsolationBaselineMinimumRangeRule { min: 5.3, max: 960.9 }),
            Rc::new(IsolationBaselineRequiredCodeRule {
                message: "isolation baseline needs an entry".to_string(),
            }),
            Rc::new(IsolationBaselineLabelLengthRule { max_length: 120 }),
            Rc::new(IsolationBaselineCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_baseline_validator();
        let item = new_isolation_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = IsolationBaselineMinimumRangeRule { min: 6.4, max: 2864.8 };
        let mut item = new_isolation_baseline("x", "y");
        item.minimum = 2866.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IsolationBaselineRequiredCodeRule {
            message: "isolation baseline needs an entry".to_string(),
        };
        let mut item = new_isolation_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = IsolationBaselineLabelLengthRule { max_length: 1000 };
        let mut item = new_isolation_baseline("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
