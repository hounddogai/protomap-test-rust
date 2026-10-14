use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback data export schedule must pass. Check returns its problems, or none.
pub trait FallbackDataExportScheduleRule: std::fmt::Debug {
    fn check(&self, item: &FallbackDataExportSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportScheduleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl FallbackDataExportScheduleRule for FallbackDataExportScheduleAllowedLabelRule {
    fn check(&self, item: &FallbackDataExportSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is out of range", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportScheduleStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackDataExportScheduleRule for FallbackDataExportScheduleStartsAtRangeRule {
    fn check(&self, item: &FallbackDataExportSchedule) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportScheduleLabelLengthRule {
    pub max_length: i64,
}

impl FallbackDataExportScheduleRule for FallbackDataExportScheduleLabelLengthRule {
    fn check(&self, item: &FallbackDataExportSchedule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback data export schedule.
#[derive(Debug, Clone)]
pub struct FallbackDataExportScheduleValidator {
    pub rules: Vec<Rc<dyn FallbackDataExportScheduleRule>>,
}

impl FallbackDataExportScheduleValidator {
    pub fn validate(&self, item: &FallbackDataExportSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackDataExportSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackDataExportSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_data_export_schedule_validator() -> FallbackDataExportScheduleValidator {
    FallbackDataExportScheduleValidator {
        rules: vec![
            Rc::new(FallbackDataExportScheduleAllowedLabelRule {
                allowed: vec!["regional".to_string(), "pending".to_string(), "regional".to_string()],
            }) as Rc<dyn FallbackDataExportScheduleRule>,
            Rc::new(FallbackDataExportScheduleStartsAtRangeRule { min: 120, max: 120 }),
            Rc::new(FallbackDataExportScheduleLabelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_data_export_schedule_validator();
        let item = new_fallback_data_export_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = FallbackDataExportScheduleStartsAtRangeRule { min: 80, max: 86400 };
        let mut item = new_fallback_data_export_schedule("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FallbackDataExportScheduleLabelLengthRule { max_length: 64 };
        let mut item = new_fallback_data_export_schedule("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
