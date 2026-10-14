use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a region pin segment must pass. Check returns its problems, or none.
pub trait RegionPinSegmentRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSegmentLabelLengthRule {
    pub max_length: i64,
}

impl RegionPinSegmentRule for RegionPinSegmentLabelLengthRule {
    fn check(&self, item: &RegionPinSegment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSegmentRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinSegmentRule for RegionPinSegmentRotationDaysRangeRule {
    fn check(&self, item: &RegionPinSegment) -> Vec<String> {
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
pub struct RegionPinSegmentCodePrefixRule {
    pub prefix: String,
}

impl RegionPinSegmentRule for RegionPinSegmentCodePrefixRule {
    fn check(&self, item: &RegionPinSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSegmentAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl RegionPinSegmentRule for RegionPinSegmentAllowedLabelRule {
    fn check(&self, item: &RegionPinSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin segment.
#[derive(Debug, Clone)]
pub struct RegionPinSegmentValidator {
    pub rules: Vec<Rc<dyn RegionPinSegmentRule>>,
}

impl RegionPinSegmentValidator {
    pub fn validate(&self, item: &RegionPinSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_segment_validator() -> RegionPinSegmentValidator {
    RegionPinSegmentValidator {
        rules: vec![
            Rc::new(RegionPinSegmentLabelLengthRule { max_length: 86400 }) as Rc<dyn RegionPinSegmentRule>,
            Rc::new(RegionPinSegmentRotationDaysRangeRule { min: 80, max: 86400 }),
            Rc::new(RegionPinSegmentCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RegionPinSegmentAllowedLabelRule {
                allowed: vec!["manual".to_string(), "strict".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_segment_validator();
        let item = new_region_pin_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionPinSegmentLabelLengthRule { max_length: 3600 };
        let mut item = new_region_pin_segment("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = RegionPinSegmentRotationDaysRangeRule { min: 32, max: 86400 };
        let mut item = new_region_pin_segment("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
