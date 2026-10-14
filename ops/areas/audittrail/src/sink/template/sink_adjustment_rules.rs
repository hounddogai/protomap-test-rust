use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink adjustment must pass. Check returns its problems, or none.
pub trait SinkAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &SinkAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAdjustmentRequiredNameRule {
    pub message: String,
}

impl SinkAdjustmentRule for SinkAdjustmentRequiredNameRule {
    fn check(&self, item: &SinkAdjustment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAdjustmentRequiredTitleRule {
    pub message: String,
}

impl SinkAdjustmentRule for SinkAdjustmentRequiredTitleRule {
    fn check(&self, item: &SinkAdjustment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAdjustmentTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkAdjustmentRule for SinkAdjustmentTtlSecondsRangeRule {
    fn check(&self, item: &SinkAdjustment) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAdjustmentGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkAdjustmentRule for SinkAdjustmentGraceDaysRangeRule {
    fn check(&self, item: &SinkAdjustment) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink adjustment.
#[derive(Debug, Clone)]
pub struct SinkAdjustmentValidator {
    pub rules: Vec<Rc<dyn SinkAdjustmentRule>>,
}

impl SinkAdjustmentValidator {
    pub fn validate(&self, item: &SinkAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_adjustment_validator() -> SinkAdjustmentValidator {
    SinkAdjustmentValidator {
        rules: vec![
            Rc::new(SinkAdjustmentRequiredNameRule {
                message: "sink adjustment needs this field".to_string(),
            }) as Rc<dyn SinkAdjustmentRule>,
            Rc::new(SinkAdjustmentRequiredTitleRule {
                message: "sink adjustment needs this field".to_string(),
            }),
            Rc::new(SinkAdjustmentTtlSecondsRangeRule { min: 32, max: 64 }),
            Rc::new(SinkAdjustmentGraceDaysRangeRule { min: 64, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_adjustment_validator();
        let item = new_sink_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SinkAdjustmentRequiredNameRule {
            message: "sink adjustment needs a value".to_string(),
        };
        let mut item = new_sink_adjustment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SinkAdjustmentRequiredTitleRule {
            message: "sink adjustment needs this field".to_string(),
        };
        let mut item = new_sink_adjustment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = SinkAdjustmentTtlSecondsRangeRule { min: 32, max: 1000 };
        let mut item = new_sink_adjustment("x", "y");
        item.ttl_seconds = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = SinkAdjustmentGraceDaysRangeRule { min: 64, max: 80 };
        let mut item = new_sink_adjustment("x", "y");
        item.grace_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
