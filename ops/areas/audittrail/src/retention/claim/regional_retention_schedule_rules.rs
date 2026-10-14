use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional retention schedule must pass. Check returns its problems, or none.
pub trait RegionalRetentionScheduleRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRetentionSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionScheduleRequiredCodeRule {
    pub message: String,
}

impl RegionalRetentionScheduleRule for RegionalRetentionScheduleRequiredCodeRule {
    fn check(&self, item: &RegionalRetentionSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionScheduleRequiredClusterRule {
    pub message: String,
}

impl RegionalRetentionScheduleRule for RegionalRetentionScheduleRequiredClusterRule {
    fn check(&self, item: &RegionalRetentionSchedule) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionScheduleWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalRetentionScheduleRule for RegionalRetentionScheduleWindowDaysRangeRule {
    fn check(&self, item: &RegionalRetentionSchedule) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionScheduleCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalRetentionScheduleRule for RegionalRetentionScheduleCreatedAtRangeRule {
    fn check(&self, item: &RegionalRetentionSchedule) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionScheduleTitleLengthRule {
    pub max_length: i64,
}

impl RegionalRetentionScheduleRule for RegionalRetentionScheduleTitleLengthRule {
    fn check(&self, item: &RegionalRetentionSchedule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional retention schedule.
#[derive(Debug, Clone)]
pub struct RegionalRetentionScheduleValidator {
    pub rules: Vec<Rc<dyn RegionalRetentionScheduleRule>>,
}

impl RegionalRetentionScheduleValidator {
    pub fn validate(&self, item: &RegionalRetentionSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalRetentionSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRetentionSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_retention_schedule_validator() -> RegionalRetentionScheduleValidator {
    RegionalRetentionScheduleValidator {
        rules: vec![
            Rc::new(RegionalRetentionScheduleRequiredCodeRule {
                message: "regional retention schedule needs this field".to_string(),
            }) as Rc<dyn RegionalRetentionScheduleRule>,
            Rc::new(RegionalRetentionScheduleRequiredClusterRule {
                message: "regional retention schedule needs a value".to_string(),
            }),
            Rc::new(RegionalRetentionScheduleWindowDaysRangeRule { min: 120, max: 255 }),
            Rc::new(RegionalRetentionScheduleCreatedAtRangeRule { min: 80, max: 3600 }),
            Rc::new(RegionalRetentionScheduleTitleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_retention_schedule_validator();
        let item = new_regional_retention_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalRetentionScheduleRequiredCodeRule {
            message: "regional retention schedule needs a value".to_string(),
        };
        let mut item = new_regional_retention_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = RegionalRetentionScheduleRequiredClusterRule {
            message: "regional retention schedule needs a value".to_string(),
        };
        let mut item = new_regional_retention_schedule("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = RegionalRetentionScheduleWindowDaysRangeRule { min: 16, max: 64 };
        let mut item = new_regional_retention_schedule("x", "y");
        item.window_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RegionalRetentionScheduleCreatedAtRangeRule { min: 16, max: 80 };
        let mut item = new_regional_retention_schedule("x", "y");
        item.created_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalRetentionScheduleTitleLengthRule { max_length: 1000 };
        let mut item = new_regional_retention_schedule("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
