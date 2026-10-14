use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention cycle must pass. Check returns its problems, or none.
pub trait RetentionCycleRule: std::fmt::Debug {
    fn check(&self, item: &RetentionCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCycleRequiredSummaryRule {
    pub message: String,
}

impl RetentionCycleRule for RetentionCycleRequiredSummaryRule {
    fn check(&self, item: &RetentionCycle) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCycleSummaryLengthRule {
    pub max_length: i64,
}

impl RetentionCycleRule for RetentionCycleSummaryLengthRule {
    fn check(&self, item: &RetentionCycle) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCycleRequiredTitleRule {
    pub message: String,
}

impl RetentionCycleRule for RetentionCycleRequiredTitleRule {
    fn check(&self, item: &RetentionCycle) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCycleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RetentionCycleRule for RetentionCycleAllowedCodeRule {
    fn check(&self, item: &RetentionCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCycleNameLengthRule {
    pub max_length: i64,
}

impl RetentionCycleRule for RetentionCycleNameLengthRule {
    fn check(&self, item: &RetentionCycle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCycleMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RetentionCycleRule for RetentionCycleMaximumRangeRule {
    fn check(&self, item: &RetentionCycle) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention cycle.
#[derive(Debug, Clone)]
pub struct RetentionCycleValidator {
    pub rules: Vec<Rc<dyn RetentionCycleRule>>,
}

impl RetentionCycleValidator {
    pub fn validate(&self, item: &RetentionCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_cycle_validator() -> RetentionCycleValidator {
    RetentionCycleValidator {
        rules: vec![
            Rc::new(RetentionCycleRequiredSummaryRule {
                message: "retention cycle needs an entry".to_string(),
            }) as Rc<dyn RetentionCycleRule>,
            Rc::new(RetentionCycleSummaryLengthRule { max_length: 120 }),
            Rc::new(RetentionCycleRequiredTitleRule {
                message: "retention cycle needs an entry".to_string(),
            }),
            Rc::new(RetentionCycleAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(RetentionCycleNameLengthRule { max_length: 64 }),
            Rc::new(RetentionCycleMaximumRangeRule { min: 6.1, max: 4500.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_cycle_validator();
        let item = new_retention_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = RetentionCycleRequiredSummaryRule {
            message: "retention cycle needs an entry".to_string(),
        };
        let mut item = new_retention_cycle("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = RetentionCycleSummaryLengthRule { max_length: 86400 };
        let mut item = new_retention_cycle("x", "y");
        item.summary = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RetentionCycleRequiredTitleRule {
            message: "retention cycle needs an entry".to_string(),
        };
        let mut item = new_retention_cycle("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RetentionCycleNameLengthRule { max_length: 1000 };
        let mut item = new_retention_cycle("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RetentionCycleMaximumRangeRule { min: 0.2, max: 3396.3 };
        let mut item = new_retention_cycle("x", "y");
        item.maximum = 3397.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
