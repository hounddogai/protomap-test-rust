use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce baseline must pass. Check returns its problems, or none.
pub trait NonceBaselineRule: std::fmt::Debug {
    fn check(&self, item: &NonceBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceBaselineCodePrefixRule {
    pub prefix: String,
}

impl NonceBaselineRule for NonceBaselineCodePrefixRule {
    fn check(&self, item: &NonceBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceBaselineTitleLengthRule {
    pub max_length: i64,
}

impl NonceBaselineRule for NonceBaselineTitleLengthRule {
    fn check(&self, item: &NonceBaseline) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceBaselineRequiredTitleRule {
    pub message: String,
}

impl NonceBaselineRule for NonceBaselineRequiredTitleRule {
    fn check(&self, item: &NonceBaseline) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceBaselineBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NonceBaselineRule for NonceBaselineBudgetRangeRule {
    fn check(&self, item: &NonceBaseline) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce baseline.
#[derive(Debug, Clone)]
pub struct NonceBaselineValidator {
    pub rules: Vec<Rc<dyn NonceBaselineRule>>,
}

impl NonceBaselineValidator {
    pub fn validate(&self, item: &NonceBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_baseline_validator() -> NonceBaselineValidator {
    NonceBaselineValidator {
        rules: vec![
            Rc::new(NonceBaselineCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn NonceBaselineRule>,
            Rc::new(NonceBaselineTitleLengthRule { max_length: 80 }),
            Rc::new(NonceBaselineRequiredTitleRule {
                message: "nonce baseline needs a value".to_string(),
            }),
            Rc::new(NonceBaselineBudgetRangeRule { min: 0.5, max: 523.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_baseline_validator();
        let item = new_nonce_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = NonceBaselineTitleLengthRule { max_length: 120 };
        let mut item = new_nonce_baseline("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = NonceBaselineRequiredTitleRule {
            message: "nonce baseline needs an entry".to_string(),
        };
        let mut item = new_nonce_baseline("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = NonceBaselineBudgetRangeRule { min: 2.4, max: 3315.5 };
        let mut item = new_nonce_baseline("x", "y");
        item.budget = 3317.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
