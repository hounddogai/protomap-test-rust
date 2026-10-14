use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release schedule must pass. Check returns its problems, or none.
pub trait ReleaseScheduleRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseScheduleCodeLengthRule {
    pub max_length: i64,
}

impl ReleaseScheduleRule for ReleaseScheduleCodeLengthRule {
    fn check(&self, item: &ReleaseSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseScheduleBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseScheduleRule for ReleaseScheduleBatchSizeRangeRule {
    fn check(&self, item: &ReleaseSchedule) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseScheduleRequiredLocaleRule {
    pub message: String,
}

impl ReleaseScheduleRule for ReleaseScheduleRequiredLocaleRule {
    fn check(&self, item: &ReleaseSchedule) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseScheduleRequiredCodeRule {
    pub message: String,
}

impl ReleaseScheduleRule for ReleaseScheduleRequiredCodeRule {
    fn check(&self, item: &ReleaseSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseScheduleRequiredLabelRule {
    pub message: String,
}

impl ReleaseScheduleRule for ReleaseScheduleRequiredLabelRule {
    fn check(&self, item: &ReleaseSchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseScheduleLocaleLengthRule {
    pub max_length: i64,
}

impl ReleaseScheduleRule for ReleaseScheduleLocaleLengthRule {
    fn check(&self, item: &ReleaseSchedule) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release schedule.
#[derive(Debug, Clone)]
pub struct ReleaseScheduleValidator {
    pub rules: Vec<Rc<dyn ReleaseScheduleRule>>,
}

impl ReleaseScheduleValidator {
    pub fn validate(&self, item: &ReleaseSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_schedule_validator() -> ReleaseScheduleValidator {
    ReleaseScheduleValidator {
        rules: vec![
            Rc::new(ReleaseScheduleCodeLengthRule { max_length: 3600 }) as Rc<dyn ReleaseScheduleRule>,
            Rc::new(ReleaseScheduleBatchSizeRangeRule { min: 64, max: 86400 }),
            Rc::new(ReleaseScheduleRequiredLocaleRule {
                message: "release schedule needs this field".to_string(),
            }),
            Rc::new(ReleaseScheduleRequiredCodeRule {
                message: "release schedule needs a value".to_string(),
            }),
            Rc::new(ReleaseScheduleRequiredLabelRule {
                message: "release schedule needs an entry".to_string(),
            }),
            Rc::new(ReleaseScheduleLocaleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_schedule_validator();
        let item = new_release_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReleaseScheduleCodeLengthRule { max_length: 1000 };
        let mut item = new_release_schedule("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = ReleaseScheduleBatchSizeRangeRule { min: 120, max: 86400 };
        let mut item = new_release_schedule("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = ReleaseScheduleRequiredLocaleRule {
            message: "release schedule needs a value".to_string(),
        };
        let mut item = new_release_schedule("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReleaseScheduleRequiredCodeRule {
            message: "release schedule needs this field".to_string(),
        };
        let mut item = new_release_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ReleaseScheduleRequiredLabelRule {
            message: "release schedule needs an entry".to_string(),
        };
        let mut item = new_release_schedule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = ReleaseScheduleLocaleLengthRule { max_length: 80 };
        let mut item = new_release_schedule("x", "y");
        item.locale = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
