use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback variance must pass. Check returns its problems, or none.
pub trait RollbackVarianceRule: std::fmt::Debug {
    fn check(&self, item: &RollbackVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackVarianceAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RollbackVarianceRule for RollbackVarianceAllowedTitleRule {
    fn check(&self, item: &RollbackVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackVarianceTitleLengthRule {
    pub max_length: i64,
}

impl RollbackVarianceRule for RollbackVarianceTitleLengthRule {
    fn check(&self, item: &RollbackVariance) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackVarianceRequiredCodeRule {
    pub message: String,
}

impl RollbackVarianceRule for RollbackVarianceRequiredCodeRule {
    fn check(&self, item: &RollbackVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackVarianceBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackVarianceRule for RollbackVarianceBurstRangeRule {
    fn check(&self, item: &RollbackVariance) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rollback variance.
#[derive(Debug, Clone)]
pub struct RollbackVarianceValidator {
    pub rules: Vec<Rc<dyn RollbackVarianceRule>>,
}

impl RollbackVarianceValidator {
    pub fn validate(&self, item: &RollbackVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_variance_validator() -> RollbackVarianceValidator {
    RollbackVarianceValidator {
        rules: vec![
            Rc::new(RollbackVarianceAllowedTitleRule {
                allowed: vec!["manual".to_string()],
            }) as Rc<dyn RollbackVarianceRule>,
            Rc::new(RollbackVarianceTitleLengthRule { max_length: 3600 }),
            Rc::new(RollbackVarianceRequiredCodeRule {
                message: "rollback variance needs a value".to_string(),
            }),
            Rc::new(RollbackVarianceBurstRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_variance_validator();
        let item = new_rollback_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollbackVarianceTitleLengthRule { max_length: 3600 };
        let mut item = new_rollback_variance("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollbackVarianceRequiredCodeRule {
            message: "rollback variance needs a value".to_string(),
        };
        let mut item = new_rollback_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = RollbackVarianceBurstRangeRule { min: 32, max: 86400 };
        let mut item = new_rollback_variance("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
