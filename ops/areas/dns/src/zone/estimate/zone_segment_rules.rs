use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone segment must pass. Check returns its problems, or none.
pub trait ZoneSegmentRule: std::fmt::Debug {
    fn check(&self, item: &ZoneSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSegmentAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ZoneSegmentRule for ZoneSegmentAmountRangeRule {
    fn check(&self, item: &ZoneSegment) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSegmentRequiredTitleRule {
    pub message: String,
}

impl ZoneSegmentRule for ZoneSegmentRequiredTitleRule {
    fn check(&self, item: &ZoneSegment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSegmentAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ZoneSegmentRule for ZoneSegmentAllowedTitleRule {
    fn check(&self, item: &ZoneSegment) -> Vec<String> {
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
pub struct ZoneSegmentTitleLengthRule {
    pub max_length: i64,
}

impl ZoneSegmentRule for ZoneSegmentTitleLengthRule {
    fn check(&self, item: &ZoneSegment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone segment.
#[derive(Debug, Clone)]
pub struct ZoneSegmentValidator {
    pub rules: Vec<Rc<dyn ZoneSegmentRule>>,
}

impl ZoneSegmentValidator {
    pub fn validate(&self, item: &ZoneSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ZoneSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_segment_validator() -> ZoneSegmentValidator {
    ZoneSegmentValidator {
        rules: vec![
            Rc::new(ZoneSegmentAmountRangeRule { min: 9.2, max: 1512.8 }) as Rc<dyn ZoneSegmentRule>,
            Rc::new(ZoneSegmentRequiredTitleRule {
                message: "zone segment needs an entry".to_string(),
            }),
            Rc::new(ZoneSegmentAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(ZoneSegmentTitleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_segment_validator();
        let item = new_zone_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = ZoneSegmentAmountRangeRule { min: 5.5, max: 1636.5 };
        let mut item = new_zone_segment("x", "y");
        item.amount = 1638.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ZoneSegmentRequiredTitleRule {
            message: "zone segment needs a value".to_string(),
        };
        let mut item = new_zone_segment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ZoneSegmentTitleLengthRule { max_length: 80 };
        let mut item = new_zone_segment("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
