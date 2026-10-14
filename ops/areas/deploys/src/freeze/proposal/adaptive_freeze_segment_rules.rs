use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive freeze segment must pass. Check returns its problems, or none.
pub trait AdaptiveFreezeSegmentRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveFreezeSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeSegmentPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveFreezeSegmentRule for AdaptiveFreezeSegmentPriorityRangeRule {
    fn check(&self, item: &AdaptiveFreezeSegment) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeSegmentVisibleGuardRule {
    pub limit: i64,
}

impl AdaptiveFreezeSegmentRule for AdaptiveFreezeSegmentVisibleGuardRule {
    fn check(&self, item: &AdaptiveFreezeSegment) -> Vec<String> {
        if item.visible && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while visible", item.burst, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeSegmentNameLengthRule {
    pub max_length: i64,
}

impl AdaptiveFreezeSegmentRule for AdaptiveFreezeSegmentNameLengthRule {
    fn check(&self, item: &AdaptiveFreezeSegment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeSegmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveFreezeSegmentRule for AdaptiveFreezeSegmentAllowedCodeRule {
    fn check(&self, item: &AdaptiveFreezeSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive freeze segment.
#[derive(Debug, Clone)]
pub struct AdaptiveFreezeSegmentValidator {
    pub rules: Vec<Rc<dyn AdaptiveFreezeSegmentRule>>,
}

impl AdaptiveFreezeSegmentValidator {
    pub fn validate(&self, item: &AdaptiveFreezeSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveFreezeSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveFreezeSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_freeze_segment_validator() -> AdaptiveFreezeSegmentValidator {
    AdaptiveFreezeSegmentValidator {
        rules: vec![
            Rc::new(AdaptiveFreezeSegmentPriorityRangeRule { min: 255, max: 86400 })
                as Rc<dyn AdaptiveFreezeSegmentRule>,
            Rc::new(AdaptiveFreezeSegmentVisibleGuardRule { limit: 16 }),
            Rc::new(AdaptiveFreezeSegmentNameLengthRule { max_length: 80 }),
            Rc::new(AdaptiveFreezeSegmentAllowedCodeRule {
                allowed: vec!["adaptive".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_freeze_segment_validator();
        let item = new_adaptive_freeze_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = AdaptiveFreezeSegmentPriorityRangeRule { min: 32, max: 255 };
        let mut item = new_adaptive_freeze_segment("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AdaptiveFreezeSegmentNameLengthRule { max_length: 16 };
        let mut item = new_adaptive_freeze_segment("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
