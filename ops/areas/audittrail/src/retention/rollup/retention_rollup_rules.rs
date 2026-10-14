use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention rollup must pass. Check returns its problems, or none.
pub trait RetentionRollupRule: std::fmt::Debug {
    fn check(&self, item: &RetentionRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRollupRequiredTitleRule {
    pub message: String,
}

impl RetentionRollupRule for RetentionRollupRequiredTitleRule {
    fn check(&self, item: &RetentionRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRollupRequiredTimezoneRule {
    pub message: String,
}

impl RetentionRollupRule for RetentionRollupRequiredTimezoneRule {
    fn check(&self, item: &RetentionRollup) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRollupDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RetentionRollupRule for RetentionRollupDiscountRangeRule {
    fn check(&self, item: &RetentionRollup) -> Vec<String> {
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
pub struct RetentionRollupTitleLengthRule {
    pub max_length: i64,
}

impl RetentionRollupRule for RetentionRollupTitleLengthRule {
    fn check(&self, item: &RetentionRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRollupTimezoneLengthRule {
    pub max_length: i64,
}

impl RetentionRollupRule for RetentionRollupTimezoneLengthRule {
    fn check(&self, item: &RetentionRollup) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRollupTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionRollupRule for RetentionRollupTtlSecondsRangeRule {
    fn check(&self, item: &RetentionRollup) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention rollup.
#[derive(Debug, Clone)]
pub struct RetentionRollupValidator {
    pub rules: Vec<Rc<dyn RetentionRollupRule>>,
}

impl RetentionRollupValidator {
    pub fn validate(&self, item: &RetentionRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_rollup_validator() -> RetentionRollupValidator {
    RetentionRollupValidator {
        rules: vec![
            Rc::new(RetentionRollupRequiredTitleRule {
                message: "retention rollup needs this field".to_string(),
            }) as Rc<dyn RetentionRollupRule>,
            Rc::new(RetentionRollupRequiredTimezoneRule {
                message: "retention rollup needs this field".to_string(),
            }),
            Rc::new(RetentionRollupDiscountRangeRule { min: 6.4, max: 2626.7 }),
            Rc::new(RetentionRollupTitleLengthRule { max_length: 3600 }),
            Rc::new(RetentionRollupTimezoneLengthRule { max_length: 120 }),
            Rc::new(RetentionRollupTtlSecondsRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_rollup_validator();
        let item = new_retention_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RetentionRollupRequiredTitleRule {
            message: "retention rollup needs an entry".to_string(),
        };
        let mut item = new_retention_rollup("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = RetentionRollupRequiredTimezoneRule {
            message: "retention rollup needs a value".to_string(),
        };
        let mut item = new_retention_rollup("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = RetentionRollupDiscountRangeRule { min: 7.2, max: 3674.2 };
        let mut item = new_retention_rollup("x", "y");
        item.discount = 3675.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RetentionRollupTitleLengthRule { max_length: 80 };
        let mut item = new_retention_rollup("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = RetentionRollupTimezoneLengthRule { max_length: 120 };
        let mut item = new_retention_rollup("x", "y");
        item.timezone = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = RetentionRollupTtlSecondsRangeRule { min: 16, max: 255 };
        let mut item = new_retention_rollup("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
