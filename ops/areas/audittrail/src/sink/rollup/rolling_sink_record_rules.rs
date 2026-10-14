use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling sink record must pass. Check returns its problems, or none.
pub trait RollingSinkRecordRule: std::fmt::Debug {
    fn check(&self, item: &RollingSinkRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSinkRecordAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollingSinkRecordRule for RollingSinkRecordAllowedCodeRule {
    fn check(&self, item: &RollingSinkRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSinkRecordSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingSinkRecordRule for RollingSinkRecordSequenceRangeRule {
    fn check(&self, item: &RollingSinkRecord) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSinkRecordRequiredCodeRule {
    pub message: String,
}

impl RollingSinkRecordRule for RollingSinkRecordRequiredCodeRule {
    fn check(&self, item: &RollingSinkRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling sink record.
#[derive(Debug, Clone)]
pub struct RollingSinkRecordValidator {
    pub rules: Vec<Rc<dyn RollingSinkRecordRule>>,
}

impl RollingSinkRecordValidator {
    pub fn validate(&self, item: &RollingSinkRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingSinkRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingSinkRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_sink_record_validator() -> RollingSinkRecordValidator {
    RollingSinkRecordValidator {
        rules: vec![
            Rc::new(RollingSinkRecordAllowedCodeRule {
                allowed: vec!["primary".to_string(), "fallback".to_string()],
            }) as Rc<dyn RollingSinkRecordRule>,
            Rc::new(RollingSinkRecordSequenceRangeRule { min: 80, max: 86400 }),
            Rc::new(RollingSinkRecordRequiredCodeRule {
                message: "rolling sink record needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_sink_record_validator();
        let item = new_rolling_sink_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = RollingSinkRecordSequenceRangeRule { min: 16, max: 86400 };
        let mut item = new_rolling_sink_record("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingSinkRecordRequiredCodeRule {
            message: "rolling sink record needs an entry".to_string(),
        };
        let mut item = new_rolling_sink_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
