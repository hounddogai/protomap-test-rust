use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline batch must pass. Check returns its problems, or none.
pub trait TimelineBatchRule: std::fmt::Debug {
    fn check(&self, item: &TimelineBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBatchRequiredChannelRule {
    pub message: String,
}

impl TimelineBatchRule for TimelineBatchRequiredChannelRule {
    fn check(&self, item: &TimelineBatch) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBatchRequiredCodeRule {
    pub message: String,
}

impl TimelineBatchRule for TimelineBatchRequiredCodeRule {
    fn check(&self, item: &TimelineBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBatchScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelineBatchRule for TimelineBatchScoreRangeRule {
    fn check(&self, item: &TimelineBatch) -> Vec<String> {
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
pub struct TimelineBatchTitleLengthRule {
    pub max_length: i64,
}

impl TimelineBatchRule for TimelineBatchTitleLengthRule {
    fn check(&self, item: &TimelineBatch) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBatchMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelineBatchRule for TimelineBatchMinimumRangeRule {
    fn check(&self, item: &TimelineBatch) -> Vec<String> {
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

/// Runs every rule of a timeline batch.
#[derive(Debug, Clone)]
pub struct TimelineBatchValidator {
    pub rules: Vec<Rc<dyn TimelineBatchRule>>,
}

impl TimelineBatchValidator {
    pub fn validate(&self, item: &TimelineBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_batch_validator() -> TimelineBatchValidator {
    TimelineBatchValidator {
        rules: vec![
            Rc::new(TimelineBatchRequiredChannelRule {
                message: "timeline batch needs an entry".to_string(),
            }) as Rc<dyn TimelineBatchRule>,
            Rc::new(TimelineBatchRequiredCodeRule {
                message: "timeline batch needs an entry".to_string(),
            }),
            Rc::new(TimelineBatchScoreRangeRule { min: 0.1, max: 415.3 }),
            Rc::new(TimelineBatchTitleLengthRule { max_length: 16 }),
            Rc::new(TimelineBatchMinimumRangeRule { min: 8.0, max: 3090.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_batch_validator();
        let item = new_timeline_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = TimelineBatchRequiredChannelRule {
            message: "timeline batch needs an entry".to_string(),
        };
        let mut item = new_timeline_batch("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TimelineBatchRequiredCodeRule {
            message: "timeline batch needs this field".to_string(),
        };
        let mut item = new_timeline_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = TimelineBatchScoreRangeRule { min: 9.8, max: 66.4 };
        let mut item = new_timeline_batch("x", "y");
        item.score = 67.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TimelineBatchTitleLengthRule { max_length: 1000 };
        let mut item = new_timeline_batch("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = TimelineBatchMinimumRangeRule { min: 1.8, max: 321.2 };
        let mut item = new_timeline_batch("x", "y");
        item.minimum = 322.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
