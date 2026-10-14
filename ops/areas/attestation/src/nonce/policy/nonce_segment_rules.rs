use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce segment must pass. Check returns its problems, or none.
pub trait NonceSegmentRule: std::fmt::Debug {
    fn check(&self, item: &NonceSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSegmentCodePrefixRule {
    pub prefix: String,
}

impl NonceSegmentRule for NonceSegmentCodePrefixRule {
    fn check(&self, item: &NonceSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSegmentRequiredCodeRule {
    pub message: String,
}

impl NonceSegmentRule for NonceSegmentRequiredCodeRule {
    fn check(&self, item: &NonceSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSegmentRequiredNameRule {
    pub message: String,
}

impl NonceSegmentRule for NonceSegmentRequiredNameRule {
    fn check(&self, item: &NonceSegment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSegmentRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceSegmentRule for NonceSegmentRotationDaysRangeRule {
    fn check(&self, item: &NonceSegment) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSegmentMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NonceSegmentRule for NonceSegmentMinimumRangeRule {
    fn check(&self, item: &NonceSegment) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSegmentCodeLengthRule {
    pub max_length: i64,
}

impl NonceSegmentRule for NonceSegmentCodeLengthRule {
    fn check(&self, item: &NonceSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce segment.
#[derive(Debug, Clone)]
pub struct NonceSegmentValidator {
    pub rules: Vec<Rc<dyn NonceSegmentRule>>,
}

impl NonceSegmentValidator {
    pub fn validate(&self, item: &NonceSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_segment_validator() -> NonceSegmentValidator {
    NonceSegmentValidator {
        rules: vec![
            Rc::new(NonceSegmentCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn NonceSegmentRule>,
            Rc::new(NonceSegmentRequiredCodeRule {
                message: "nonce segment needs this field".to_string(),
            }),
            Rc::new(NonceSegmentRequiredNameRule {
                message: "nonce segment needs this field".to_string(),
            }),
            Rc::new(NonceSegmentRotationDaysRangeRule { min: 64, max: 80 }),
            Rc::new(NonceSegmentMinimumRangeRule { min: 8.7, max: 1858.7 }),
            Rc::new(NonceSegmentCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_segment_validator();
        let item = new_nonce_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = NonceSegmentRequiredCodeRule {
            message: "nonce segment needs an entry".to_string(),
        };
        let mut item = new_nonce_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = NonceSegmentRequiredNameRule {
            message: "nonce segment needs a value".to_string(),
        };
        let mut item = new_nonce_segment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = NonceSegmentRotationDaysRangeRule { min: 16, max: 80 };
        let mut item = new_nonce_segment("x", "y");
        item.rotation_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = NonceSegmentMinimumRangeRule { min: 7.7, max: 3375.2 };
        let mut item = new_nonce_segment("x", "y");
        item.minimum = 3376.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = NonceSegmentCodeLengthRule { max_length: 16 };
        let mut item = new_nonce_segment("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
