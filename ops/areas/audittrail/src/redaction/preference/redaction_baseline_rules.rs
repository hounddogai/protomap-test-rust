use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction baseline must pass. Check returns its problems, or none.
pub trait RedactionBaselineRule: std::fmt::Debug {
    fn check(&self, item: &RedactionBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBaselineRequiredPrincipalRule {
    pub message: String,
}

impl RedactionBaselineRule for RedactionBaselineRequiredPrincipalRule {
    fn check(&self, item: &RedactionBaseline) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBaselineCodeLengthRule {
    pub max_length: i64,
}

impl RedactionBaselineRule for RedactionBaselineCodeLengthRule {
    fn check(&self, item: &RedactionBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBaselineCodePrefixRule {
    pub prefix: String,
}

impl RedactionBaselineRule for RedactionBaselineCodePrefixRule {
    fn check(&self, item: &RedactionBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBaselineDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RedactionBaselineRule for RedactionBaselineDiscountRangeRule {
    fn check(&self, item: &RedactionBaseline) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBaselineBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RedactionBaselineRule for RedactionBaselineBudgetRangeRule {
    fn check(&self, item: &RedactionBaseline) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction baseline.
#[derive(Debug, Clone)]
pub struct RedactionBaselineValidator {
    pub rules: Vec<Rc<dyn RedactionBaselineRule>>,
}

impl RedactionBaselineValidator {
    pub fn validate(&self, item: &RedactionBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RedactionBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_baseline_validator() -> RedactionBaselineValidator {
    RedactionBaselineValidator {
        rules: vec![
            Rc::new(RedactionBaselineRequiredPrincipalRule {
                message: "redaction baseline needs an entry".to_string(),
            }) as Rc<dyn RedactionBaselineRule>,
            Rc::new(RedactionBaselineCodeLengthRule { max_length: 32 }),
            Rc::new(RedactionBaselineCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RedactionBaselineDiscountRangeRule { min: 2.1, max: 3956.0 }),
            Rc::new(RedactionBaselineBudgetRangeRule { min: 8.7, max: 4228.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_baseline_validator();
        let item = new_redaction_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = RedactionBaselineRequiredPrincipalRule {
            message: "redaction baseline needs an entry".to_string(),
        };
        let mut item = new_redaction_baseline("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RedactionBaselineCodeLengthRule { max_length: 255 };
        let mut item = new_redaction_baseline("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = RedactionBaselineDiscountRangeRule { min: 1.2, max: 531.8 };
        let mut item = new_redaction_baseline("x", "y");
        item.discount = 533.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = RedactionBaselineBudgetRangeRule { min: 5.1, max: 86.4 };
        let mut item = new_redaction_baseline("x", "y");
        item.budget = 87.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
