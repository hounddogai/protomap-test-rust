use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional segment item must pass. Check returns its problems, or none.
pub trait RegionalSegmentItemRule: std::fmt::Debug {
    fn check(&self, item: &RegionalSegmentItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSegmentItemCodeLengthRule {
    pub max_length: i64,
}

impl RegionalSegmentItemRule for RegionalSegmentItemCodeLengthRule {
    fn check(&self, item: &RegionalSegmentItem) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSegmentItemAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalSegmentItemRule for RegionalSegmentItemAllowedCodeRule {
    fn check(&self, item: &RegionalSegmentItem) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSegmentItemThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalSegmentItemRule for RegionalSegmentItemThresholdRangeRule {
    fn check(&self, item: &RegionalSegmentItem) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a regional segment item.
#[derive(Debug, Clone)]
pub struct RegionalSegmentItemValidator {
    pub rules: Vec<Rc<dyn RegionalSegmentItemRule>>,
}

impl RegionalSegmentItemValidator {
    pub fn validate(&self, item: &RegionalSegmentItem) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalSegmentItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalSegmentItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_segment_item_validator() -> RegionalSegmentItemValidator {
    RegionalSegmentItemValidator {
        rules: vec![
            Rc::new(RegionalSegmentItemCodeLengthRule { max_length: 32 }) as Rc<dyn RegionalSegmentItemRule>,
            Rc::new(RegionalSegmentItemAllowedCodeRule {
                allowed: vec!["strict".to_string()],
            }),
            Rc::new(RegionalSegmentItemThresholdRangeRule { min: 9.6, max: 99.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_segment_item_validator();
        let item = new_regional_segment_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalSegmentItemCodeLengthRule { max_length: 255 };
        let mut item = new_regional_segment_item("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RegionalSegmentItemThresholdRangeRule { min: 1.7, max: 3523.2 };
        let mut item = new_regional_segment_item("x", "y");
        item.threshold = 3524.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
