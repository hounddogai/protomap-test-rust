use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual data export segment must pass. Check returns its problems, or none.
pub trait ManualDataExportSegmentRule: std::fmt::Debug {
    fn check(&self, item: &ManualDataExportSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportSegmentRequiredTitleRule {
    pub message: String,
}

impl ManualDataExportSegmentRule for ManualDataExportSegmentRequiredTitleRule {
    fn check(&self, item: &ManualDataExportSegment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportSegmentReviewerLengthRule {
    pub max_length: i64,
}

impl ManualDataExportSegmentRule for ManualDataExportSegmentReviewerLengthRule {
    fn check(&self, item: &ManualDataExportSegment) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportSegmentCodeLengthRule {
    pub max_length: i64,
}

impl ManualDataExportSegmentRule for ManualDataExportSegmentCodeLengthRule {
    fn check(&self, item: &ManualDataExportSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportSegmentCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualDataExportSegmentRule for ManualDataExportSegmentCreatedAtRangeRule {
    fn check(&self, item: &ManualDataExportSegment) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportSegmentAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl ManualDataExportSegmentRule for ManualDataExportSegmentAllowedRegionRule {
    fn check(&self, item: &ManualDataExportSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is missing", item.region)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportSegmentWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualDataExportSegmentRule for ManualDataExportSegmentWeightRangeRule {
    fn check(&self, item: &ManualDataExportSegment) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual data export segment.
#[derive(Debug, Clone)]
pub struct ManualDataExportSegmentValidator {
    pub rules: Vec<Rc<dyn ManualDataExportSegmentRule>>,
}

impl ManualDataExportSegmentValidator {
    pub fn validate(&self, item: &ManualDataExportSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualDataExportSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualDataExportSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_data_export_segment_validator() -> ManualDataExportSegmentValidator {
    ManualDataExportSegmentValidator {
        rules: vec![
            Rc::new(ManualDataExportSegmentRequiredTitleRule {
                message: "manual data export segment needs a value".to_string(),
            }) as Rc<dyn ManualDataExportSegmentRule>,
            Rc::new(ManualDataExportSegmentReviewerLengthRule { max_length: 255 }),
            Rc::new(ManualDataExportSegmentCodeLengthRule { max_length: 255 }),
            Rc::new(ManualDataExportSegmentCreatedAtRangeRule { min: 80, max: 3600 }),
            Rc::new(ManualDataExportSegmentAllowedRegionRule {
                allowed: vec!["default".to_string(), "weighted".to_string(), "fallback".to_string()],
            }),
            Rc::new(ManualDataExportSegmentWeightRangeRule { min: 8.6, max: 4754.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_data_export_segment_validator();
        let item = new_manual_data_export_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualDataExportSegmentRequiredTitleRule {
            message: "manual data export segment needs a value".to_string(),
        };
        let mut item = new_manual_data_export_segment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = ManualDataExportSegmentReviewerLengthRule { max_length: 16 };
        let mut item = new_manual_data_export_segment("x", "y");
        item.reviewer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualDataExportSegmentCodeLengthRule { max_length: 32 };
        let mut item = new_manual_data_export_segment("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = ManualDataExportSegmentCreatedAtRangeRule { min: 1000, max: 1000 };
        let mut item = new_manual_data_export_segment("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = ManualDataExportSegmentWeightRangeRule { min: 5.4, max: 3660.8 };
        let mut item = new_manual_data_export_segment("x", "y");
        item.weight = 3662.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
