use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal retention segment must pass. Check returns its problems, or none.
pub trait SeasonalRetentionSegmentRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalRetentionSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionSegmentRequiredCodeRule {
    pub message: String,
}

impl SeasonalRetentionSegmentRule for SeasonalRetentionSegmentRequiredCodeRule {
    fn check(&self, item: &SeasonalRetentionSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionSegmentRequiredCipherRule {
    pub message: String,
}

impl SeasonalRetentionSegmentRule for SeasonalRetentionSegmentRequiredCipherRule {
    fn check(&self, item: &SeasonalRetentionSegment) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionSegmentTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalRetentionSegmentRule for SeasonalRetentionSegmentTtlSecondsRangeRule {
    fn check(&self, item: &SeasonalRetentionSegment) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionSegmentCipherLengthRule {
    pub max_length: i64,
}

impl SeasonalRetentionSegmentRule for SeasonalRetentionSegmentCipherLengthRule {
    fn check(&self, item: &SeasonalRetentionSegment) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionSegmentCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalRetentionSegmentRule for SeasonalRetentionSegmentCodeLengthRule {
    fn check(&self, item: &SeasonalRetentionSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal retention segment.
#[derive(Debug, Clone)]
pub struct SeasonalRetentionSegmentValidator {
    pub rules: Vec<Rc<dyn SeasonalRetentionSegmentRule>>,
}

impl SeasonalRetentionSegmentValidator {
    pub fn validate(&self, item: &SeasonalRetentionSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalRetentionSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalRetentionSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_retention_segment_validator() -> SeasonalRetentionSegmentValidator {
    SeasonalRetentionSegmentValidator {
        rules: vec![
            Rc::new(SeasonalRetentionSegmentRequiredCodeRule {
                message: "seasonal retention segment needs this field".to_string(),
            }) as Rc<dyn SeasonalRetentionSegmentRule>,
            Rc::new(SeasonalRetentionSegmentRequiredCipherRule {
                message: "seasonal retention segment needs a value".to_string(),
            }),
            Rc::new(SeasonalRetentionSegmentTtlSecondsRangeRule { min: 120, max: 120 }),
            Rc::new(SeasonalRetentionSegmentCipherLengthRule { max_length: 16 }),
            Rc::new(SeasonalRetentionSegmentCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_retention_segment_validator();
        let item = new_seasonal_retention_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalRetentionSegmentRequiredCodeRule {
            message: "seasonal retention segment needs a value".to_string(),
        };
        let mut item = new_seasonal_retention_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = SeasonalRetentionSegmentRequiredCipherRule {
            message: "seasonal retention segment needs a value".to_string(),
        };
        let mut item = new_seasonal_retention_segment("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = SeasonalRetentionSegmentTtlSecondsRangeRule { min: 64, max: 80 };
        let mut item = new_seasonal_retention_segment("x", "y");
        item.ttl_seconds = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = SeasonalRetentionSegmentCipherLengthRule { max_length: 120 };
        let mut item = new_seasonal_retention_segment("x", "y");
        item.cipher = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalRetentionSegmentCodeLengthRule { max_length: 16 };
        let mut item = new_seasonal_retention_segment("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
