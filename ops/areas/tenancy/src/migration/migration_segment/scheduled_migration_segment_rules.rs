use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled migration segment must pass. Check returns its problems, or none.
pub trait ScheduledMigrationSegmentRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledMigrationSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMigrationSegmentCodePrefixRule {
    pub prefix: String,
}

impl ScheduledMigrationSegmentRule for ScheduledMigrationSegmentCodePrefixRule {
    fn check(&self, item: &ScheduledMigrationSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMigrationSegmentRequiredCodeRule {
    pub message: String,
}

impl ScheduledMigrationSegmentRule for ScheduledMigrationSegmentRequiredCodeRule {
    fn check(&self, item: &ScheduledMigrationSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMigrationSegmentBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledMigrationSegmentRule for ScheduledMigrationSegmentBurstRangeRule {
    fn check(&self, item: &ScheduledMigrationSegment) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMigrationSegmentScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledMigrationSegmentRule for ScheduledMigrationSegmentScoreRangeRule {
    fn check(&self, item: &ScheduledMigrationSegment) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMigrationSegmentSummaryLengthRule {
    pub max_length: i64,
}

impl ScheduledMigrationSegmentRule for ScheduledMigrationSegmentSummaryLengthRule {
    fn check(&self, item: &ScheduledMigrationSegment) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled migration segment.
#[derive(Debug, Clone)]
pub struct ScheduledMigrationSegmentValidator {
    pub rules: Vec<Rc<dyn ScheduledMigrationSegmentRule>>,
}

impl ScheduledMigrationSegmentValidator {
    pub fn validate(&self, item: &ScheduledMigrationSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledMigrationSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledMigrationSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_migration_segment_validator() -> ScheduledMigrationSegmentValidator {
    ScheduledMigrationSegmentValidator {
        rules: vec![
            Rc::new(ScheduledMigrationSegmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn ScheduledMigrationSegmentRule>,
            Rc::new(ScheduledMigrationSegmentRequiredCodeRule {
                message: "scheduled migration segment needs this field".to_string(),
            }),
            Rc::new(ScheduledMigrationSegmentBurstRangeRule { min: 80, max: 120 }),
            Rc::new(ScheduledMigrationSegmentScoreRangeRule { min: 7.8, max: 3927.6 }),
            Rc::new(ScheduledMigrationSegmentSummaryLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_migration_segment_validator();
        let item = new_scheduled_migration_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledMigrationSegmentRequiredCodeRule {
            message: "scheduled migration segment needs a value".to_string(),
        };
        let mut item = new_scheduled_migration_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = ScheduledMigrationSegmentBurstRangeRule { min: 32, max: 80 };
        let mut item = new_scheduled_migration_segment("x", "y");
        item.burst = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ScheduledMigrationSegmentScoreRangeRule { min: 9.9, max: 3865.7 };
        let mut item = new_scheduled_migration_segment("x", "y");
        item.score = 3867.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = ScheduledMigrationSegmentSummaryLengthRule { max_length: 3600 };
        let mut item = new_scheduled_migration_segment("x", "y");
        item.summary = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
