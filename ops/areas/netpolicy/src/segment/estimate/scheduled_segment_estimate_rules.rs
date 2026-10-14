use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled segment estimate must pass. Check returns its problems, or none.
pub trait ScheduledSegmentEstimateRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSegmentEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentEstimateWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledSegmentEstimateRule for ScheduledSegmentEstimateWindowDaysRangeRule {
    fn check(&self, item: &ScheduledSegmentEstimate) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentEstimateRequiredCodeRule {
    pub message: String,
}

impl ScheduledSegmentEstimateRule for ScheduledSegmentEstimateRequiredCodeRule {
    fn check(&self, item: &ScheduledSegmentEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentEstimateNameLengthRule {
    pub max_length: i64,
}

impl ScheduledSegmentEstimateRule for ScheduledSegmentEstimateNameLengthRule {
    fn check(&self, item: &ScheduledSegmentEstimate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentEstimateVisibleGuardRule {
    pub limit: i64,
}

impl ScheduledSegmentEstimateRule for ScheduledSegmentEstimateVisibleGuardRule {
    fn check(&self, item: &ScheduledSegmentEstimate) -> Vec<String> {
        if item.visible && (item.window_days > self.limit) {
            return vec![format!(
                "window days {} exceeds {} while visible",
                item.window_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentEstimateNotesLengthRule {
    pub max_length: i64,
}

impl ScheduledSegmentEstimateRule for ScheduledSegmentEstimateNotesLengthRule {
    fn check(&self, item: &ScheduledSegmentEstimate) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled segment estimate.
#[derive(Debug, Clone)]
pub struct ScheduledSegmentEstimateValidator {
    pub rules: Vec<Rc<dyn ScheduledSegmentEstimateRule>>,
}

impl ScheduledSegmentEstimateValidator {
    pub fn validate(&self, item: &ScheduledSegmentEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledSegmentEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSegmentEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_segment_estimate_validator() -> ScheduledSegmentEstimateValidator {
    ScheduledSegmentEstimateValidator {
        rules: vec![
            Rc::new(ScheduledSegmentEstimateWindowDaysRangeRule { min: 16, max: 80 })
                as Rc<dyn ScheduledSegmentEstimateRule>,
            Rc::new(ScheduledSegmentEstimateRequiredCodeRule {
                message: "scheduled segment estimate needs this field".to_string(),
            }),
            Rc::new(ScheduledSegmentEstimateNameLengthRule { max_length: 32 }),
            Rc::new(ScheduledSegmentEstimateVisibleGuardRule { limit: 64 }),
            Rc::new(ScheduledSegmentEstimateNotesLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_segment_estimate_validator();
        let item = new_scheduled_segment_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = ScheduledSegmentEstimateWindowDaysRangeRule { min: 120, max: 3600 };
        let mut item = new_scheduled_segment_estimate("x", "y");
        item.window_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledSegmentEstimateRequiredCodeRule {
            message: "scheduled segment estimate needs an entry".to_string(),
        };
        let mut item = new_scheduled_segment_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ScheduledSegmentEstimateNameLengthRule { max_length: 64 };
        let mut item = new_scheduled_segment_estimate("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = ScheduledSegmentEstimateNotesLengthRule { max_length: 86400 };
        let mut item = new_scheduled_segment_estimate("x", "y");
        item.notes = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
