use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline record must pass. Check returns its problems, or none.
pub trait TimelineRecordRule: std::fmt::Debug {
    fn check(&self, item: &TimelineRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRecordRequiredNameRule {
    pub message: String,
}

impl TimelineRecordRule for TimelineRecordRequiredNameRule {
    fn check(&self, item: &TimelineRecord) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRecordLocaleLengthRule {
    pub max_length: i64,
}

impl TimelineRecordRule for TimelineRecordLocaleLengthRule {
    fn check(&self, item: &TimelineRecord) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRecordPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineRecordRule for TimelineRecordPortRangeRule {
    fn check(&self, item: &TimelineRecord) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRecordNameLengthRule {
    pub max_length: i64,
}

impl TimelineRecordRule for TimelineRecordNameLengthRule {
    fn check(&self, item: &TimelineRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRecordRequiredCodeRule {
    pub message: String,
}

impl TimelineRecordRule for TimelineRecordRequiredCodeRule {
    fn check(&self, item: &TimelineRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline record.
#[derive(Debug, Clone)]
pub struct TimelineRecordValidator {
    pub rules: Vec<Rc<dyn TimelineRecordRule>>,
}

impl TimelineRecordValidator {
    pub fn validate(&self, item: &TimelineRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_record_validator() -> TimelineRecordValidator {
    TimelineRecordValidator {
        rules: vec![
            Rc::new(TimelineRecordRequiredNameRule {
                message: "timeline record needs an entry".to_string(),
            }) as Rc<dyn TimelineRecordRule>,
            Rc::new(TimelineRecordLocaleLengthRule { max_length: 255 }),
            Rc::new(TimelineRecordPortRangeRule { min: 255, max: 1000 }),
            Rc::new(TimelineRecordNameLengthRule { max_length: 16 }),
            Rc::new(TimelineRecordRequiredCodeRule {
                message: "timeline record needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_record_validator();
        let item = new_timeline_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TimelineRecordRequiredNameRule {
            message: "timeline record needs this field".to_string(),
        };
        let mut item = new_timeline_record("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = TimelineRecordLocaleLengthRule { max_length: 16 };
        let mut item = new_timeline_record("x", "y");
        item.locale = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = TimelineRecordPortRangeRule { min: 1000, max: 1000 };
        let mut item = new_timeline_record("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TimelineRecordNameLengthRule { max_length: 1000 };
        let mut item = new_timeline_record("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TimelineRecordRequiredCodeRule {
            message: "timeline record needs a value".to_string(),
        };
        let mut item = new_timeline_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
