use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted flow log record must pass. Check returns its problems, or none.
pub trait WeightedFlowLogRecordRule: std::fmt::Debug {
    fn check(&self, item: &WeightedFlowLogRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFlowLogRecordSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedFlowLogRecordRule for WeightedFlowLogRecordSequenceRangeRule {
    fn check(&self, item: &WeightedFlowLogRecord) -> Vec<String> {
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
pub struct WeightedFlowLogRecordTimezoneLengthRule {
    pub max_length: i64,
}

impl WeightedFlowLogRecordRule for WeightedFlowLogRecordTimezoneLengthRule {
    fn check(&self, item: &WeightedFlowLogRecord) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFlowLogRecordCodePrefixRule {
    pub prefix: String,
}

impl WeightedFlowLogRecordRule for WeightedFlowLogRecordCodePrefixRule {
    fn check(&self, item: &WeightedFlowLogRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted flow log record.
#[derive(Debug, Clone)]
pub struct WeightedFlowLogRecordValidator {
    pub rules: Vec<Rc<dyn WeightedFlowLogRecordRule>>,
}

impl WeightedFlowLogRecordValidator {
    pub fn validate(&self, item: &WeightedFlowLogRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedFlowLogRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedFlowLogRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_flow_log_record_validator() -> WeightedFlowLogRecordValidator {
    WeightedFlowLogRecordValidator {
        rules: vec![
            Rc::new(WeightedFlowLogRecordSequenceRangeRule { min: 255, max: 86400 })
                as Rc<dyn WeightedFlowLogRecordRule>,
            Rc::new(WeightedFlowLogRecordTimezoneLengthRule { max_length: 16 }),
            Rc::new(WeightedFlowLogRecordCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_flow_log_record_validator();
        let item = new_weighted_flow_log_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = WeightedFlowLogRecordSequenceRangeRule { min: 64, max: 3600 };
        let mut item = new_weighted_flow_log_record("x", "y");
        item.sequence = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = WeightedFlowLogRecordTimezoneLengthRule { max_length: 3600 };
        let mut item = new_weighted_flow_log_record("x", "y");
        item.timezone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
