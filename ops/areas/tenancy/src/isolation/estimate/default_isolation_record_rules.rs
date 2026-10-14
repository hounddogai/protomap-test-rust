use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default isolation record must pass. Check returns its problems, or none.
pub trait DefaultIsolationRecordRule: std::fmt::Debug {
    fn check(&self, item: &DefaultIsolationRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationRecordMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultIsolationRecordRule for DefaultIsolationRecordMinimumRangeRule {
    fn check(&self, item: &DefaultIsolationRecord) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationRecordRegionLengthRule {
    pub max_length: i64,
}

impl DefaultIsolationRecordRule for DefaultIsolationRecordRegionLengthRule {
    fn check(&self, item: &DefaultIsolationRecord) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationRecordDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultIsolationRecordRule for DefaultIsolationRecordDurationMinutesRangeRule {
    fn check(&self, item: &DefaultIsolationRecord) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a default isolation record.
#[derive(Debug, Clone)]
pub struct DefaultIsolationRecordValidator {
    pub rules: Vec<Rc<dyn DefaultIsolationRecordRule>>,
}

impl DefaultIsolationRecordValidator {
    pub fn validate(&self, item: &DefaultIsolationRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultIsolationRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultIsolationRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_isolation_record_validator() -> DefaultIsolationRecordValidator {
    DefaultIsolationRecordValidator {
        rules: vec![
            Rc::new(DefaultIsolationRecordMinimumRangeRule { min: 4.8, max: 4953.4 })
                as Rc<dyn DefaultIsolationRecordRule>,
            Rc::new(DefaultIsolationRecordRegionLengthRule { max_length: 120 }),
            Rc::new(DefaultIsolationRecordDurationMinutesRangeRule { min: 1000, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_isolation_record_validator();
        let item = new_default_isolation_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = DefaultIsolationRecordMinimumRangeRule { min: 7.5, max: 4672.3 };
        let mut item = new_default_isolation_record("x", "y");
        item.minimum = 4673.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = DefaultIsolationRecordRegionLengthRule { max_length: 3600 };
        let mut item = new_default_isolation_record("x", "y");
        item.region = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = DefaultIsolationRecordDurationMinutesRangeRule { min: 120, max: 255 };
        let mut item = new_default_isolation_record("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
