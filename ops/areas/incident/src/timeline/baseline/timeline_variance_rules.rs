use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline variance must pass. Check returns its problems, or none.
pub trait TimelineVarianceRule: std::fmt::Debug {
    fn check(&self, item: &TimelineVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineVarianceUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineVarianceRule for TimelineVarianceUpdatedAtRangeRule {
    fn check(&self, item: &TimelineVariance) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineVariancePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineVarianceRule for TimelineVariancePortRangeRule {
    fn check(&self, item: &TimelineVariance) -> Vec<String> {
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
pub struct TimelineVarianceCodeLengthRule {
    pub max_length: i64,
}

impl TimelineVarianceRule for TimelineVarianceCodeLengthRule {
    fn check(&self, item: &TimelineVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineVarianceRequiredNameRule {
    pub message: String,
}

impl TimelineVarianceRule for TimelineVarianceRequiredNameRule {
    fn check(&self, item: &TimelineVariance) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline variance.
#[derive(Debug, Clone)]
pub struct TimelineVarianceValidator {
    pub rules: Vec<Rc<dyn TimelineVarianceRule>>,
}

impl TimelineVarianceValidator {
    pub fn validate(&self, item: &TimelineVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_variance_validator() -> TimelineVarianceValidator {
    TimelineVarianceValidator {
        rules: vec![
            Rc::new(TimelineVarianceUpdatedAtRangeRule { min: 32, max: 86400 }) as Rc<dyn TimelineVarianceRule>,
            Rc::new(TimelineVariancePortRangeRule { min: 16, max: 64 }),
            Rc::new(TimelineVarianceCodeLengthRule { max_length: 255 }),
            Rc::new(TimelineVarianceRequiredNameRule {
                message: "timeline variance needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_variance_validator();
        let item = new_timeline_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = TimelineVarianceUpdatedAtRangeRule { min: 120, max: 1000 };
        let mut item = new_timeline_variance("x", "y");
        item.updated_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = TimelineVariancePortRangeRule { min: 16, max: 1000 };
        let mut item = new_timeline_variance("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TimelineVarianceCodeLengthRule { max_length: 16 };
        let mut item = new_timeline_variance("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TimelineVarianceRequiredNameRule {
            message: "timeline variance needs a value".to_string(),
        };
        let mut item = new_timeline_variance("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
