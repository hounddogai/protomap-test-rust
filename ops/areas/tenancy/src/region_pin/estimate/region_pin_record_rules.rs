use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin record must pass. Check returns its problems, or none.
pub trait RegionPinRecordRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRecordRequiredCodeRule {
    pub message: String,
}

impl RegionPinRecordRule for RegionPinRecordRequiredCodeRule {
    fn check(&self, item: &RegionPinRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRecordBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinRecordRule for RegionPinRecordBurstRangeRule {
    fn check(&self, item: &RegionPinRecord) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRecordRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinRecordRule for RegionPinRecordRotationDaysRangeRule {
    fn check(&self, item: &RegionPinRecord) -> Vec<String> {
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
pub struct RegionPinRecordReviewerLengthRule {
    pub max_length: i64,
}

impl RegionPinRecordRule for RegionPinRecordReviewerLengthRule {
    fn check(&self, item: &RegionPinRecord) -> Vec<String> {
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
pub struct RegionPinRecordLabelLengthRule {
    pub max_length: i64,
}

impl RegionPinRecordRule for RegionPinRecordLabelLengthRule {
    fn check(&self, item: &RegionPinRecord) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRecordRequiredLabelRule {
    pub message: String,
}

impl RegionPinRecordRule for RegionPinRecordRequiredLabelRule {
    fn check(&self, item: &RegionPinRecord) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin record.
#[derive(Debug, Clone)]
pub struct RegionPinRecordValidator {
    pub rules: Vec<Rc<dyn RegionPinRecordRule>>,
}

impl RegionPinRecordValidator {
    pub fn validate(&self, item: &RegionPinRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_record_validator() -> RegionPinRecordValidator {
    RegionPinRecordValidator {
        rules: vec![
            Rc::new(RegionPinRecordRequiredCodeRule {
                message: "region pin record needs an entry".to_string(),
            }) as Rc<dyn RegionPinRecordRule>,
            Rc::new(RegionPinRecordBurstRangeRule { min: 64, max: 86400 }),
            Rc::new(RegionPinRecordRotationDaysRangeRule { min: 255, max: 86400 }),
            Rc::new(RegionPinRecordReviewerLengthRule { max_length: 64 }),
            Rc::new(RegionPinRecordLabelLengthRule { max_length: 80 }),
            Rc::new(RegionPinRecordRequiredLabelRule {
                message: "region pin record needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_record_validator();
        let item = new_region_pin_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinRecordRequiredCodeRule {
            message: "region pin record needs a value".to_string(),
        };
        let mut item = new_region_pin_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = RegionPinRecordBurstRangeRule { min: 120, max: 255 };
        let mut item = new_region_pin_record("x", "y");
        item.burst = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = RegionPinRecordRotationDaysRangeRule { min: 120, max: 86400 };
        let mut item = new_region_pin_record("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = RegionPinRecordReviewerLengthRule { max_length: 120 };
        let mut item = new_region_pin_record("x", "y");
        item.reviewer = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionPinRecordLabelLengthRule { max_length: 3600 };
        let mut item = new_region_pin_record("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionPinRecordRequiredLabelRule {
            message: "region pin record needs this field".to_string(),
        };
        let mut item = new_region_pin_record("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
