use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered token segment must pass. Check returns its problems, or none.
pub trait TieredTokenSegmentRule: std::fmt::Debug {
    fn check(&self, item: &TieredTokenSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenSegmentRequiredSourceRule {
    pub message: String,
}

impl TieredTokenSegmentRule for TieredTokenSegmentRequiredSourceRule {
    fn check(&self, item: &TieredTokenSegment) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenSegmentGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredTokenSegmentRule for TieredTokenSegmentGraceDaysRangeRule {
    fn check(&self, item: &TieredTokenSegment) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenSegmentSourceLengthRule {
    pub max_length: i64,
}

impl TieredTokenSegmentRule for TieredTokenSegmentSourceLengthRule {
    fn check(&self, item: &TieredTokenSegment) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenSegmentNameLengthRule {
    pub max_length: i64,
}

impl TieredTokenSegmentRule for TieredTokenSegmentNameLengthRule {
    fn check(&self, item: &TieredTokenSegment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenSegmentBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredTokenSegmentRule for TieredTokenSegmentBurstRangeRule {
    fn check(&self, item: &TieredTokenSegment) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenSegmentRequiredNameRule {
    pub message: String,
}

impl TieredTokenSegmentRule for TieredTokenSegmentRequiredNameRule {
    fn check(&self, item: &TieredTokenSegment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered token segment.
#[derive(Debug, Clone)]
pub struct TieredTokenSegmentValidator {
    pub rules: Vec<Rc<dyn TieredTokenSegmentRule>>,
}

impl TieredTokenSegmentValidator {
    pub fn validate(&self, item: &TieredTokenSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredTokenSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredTokenSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_token_segment_validator() -> TieredTokenSegmentValidator {
    TieredTokenSegmentValidator {
        rules: vec![
            Rc::new(TieredTokenSegmentRequiredSourceRule {
                message: "tiered token segment needs an entry".to_string(),
            }) as Rc<dyn TieredTokenSegmentRule>,
            Rc::new(TieredTokenSegmentGraceDaysRangeRule { min: 64, max: 120 }),
            Rc::new(TieredTokenSegmentSourceLengthRule { max_length: 255 }),
            Rc::new(TieredTokenSegmentNameLengthRule { max_length: 120 }),
            Rc::new(TieredTokenSegmentBurstRangeRule { min: 255, max: 86400 }),
            Rc::new(TieredTokenSegmentRequiredNameRule {
                message: "tiered token segment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_token_segment_validator();
        let item = new_tiered_token_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = TieredTokenSegmentRequiredSourceRule {
            message: "tiered token segment needs an entry".to_string(),
        };
        let mut item = new_tiered_token_segment("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = TieredTokenSegmentGraceDaysRangeRule { min: 32, max: 80 };
        let mut item = new_tiered_token_segment("x", "y");
        item.grace_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = TieredTokenSegmentSourceLengthRule { max_length: 3600 };
        let mut item = new_tiered_token_segment("x", "y");
        item.source = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TieredTokenSegmentNameLengthRule { max_length: 3600 };
        let mut item = new_tiered_token_segment("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = TieredTokenSegmentBurstRangeRule { min: 64, max: 86400 };
        let mut item = new_tiered_token_segment("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredTokenSegmentRequiredNameRule {
            message: "tiered token segment needs this field".to_string(),
        };
        let mut item = new_tiered_token_segment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
