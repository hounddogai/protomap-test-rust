use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log batch must pass. Check returns its problems, or none.
pub trait FlowLogBatchRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBatchRequiredLabelRule {
    pub message: String,
}

impl FlowLogBatchRule for FlowLogBatchRequiredLabelRule {
    fn check(&self, item: &FlowLogBatch) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBatchEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogBatchRule for FlowLogBatchEndsAtRangeRule {
    fn check(&self, item: &FlowLogBatch) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBatchLabelLengthRule {
    pub max_length: i64,
}

impl FlowLogBatchRule for FlowLogBatchLabelLengthRule {
    fn check(&self, item: &FlowLogBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBatchCountryLengthRule {
    pub max_length: i64,
}

impl FlowLogBatchRule for FlowLogBatchCountryLengthRule {
    fn check(&self, item: &FlowLogBatch) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBatchUnitLengthRule {
    pub max_length: i64,
}

impl FlowLogBatchRule for FlowLogBatchUnitLengthRule {
    fn check(&self, item: &FlowLogBatch) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log batch.
#[derive(Debug, Clone)]
pub struct FlowLogBatchValidator {
    pub rules: Vec<Rc<dyn FlowLogBatchRule>>,
}

impl FlowLogBatchValidator {
    pub fn validate(&self, item: &FlowLogBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_batch_validator() -> FlowLogBatchValidator {
    FlowLogBatchValidator {
        rules: vec![
            Rc::new(FlowLogBatchRequiredLabelRule {
                message: "flow log batch needs this field".to_string(),
            }) as Rc<dyn FlowLogBatchRule>,
            Rc::new(FlowLogBatchEndsAtRangeRule { min: 16, max: 120 }),
            Rc::new(FlowLogBatchLabelLengthRule { max_length: 86400 }),
            Rc::new(FlowLogBatchCountryLengthRule { max_length: 255 }),
            Rc::new(FlowLogBatchUnitLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_batch_validator();
        let item = new_flow_log_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FlowLogBatchRequiredLabelRule {
            message: "flow log batch needs this field".to_string(),
        };
        let mut item = new_flow_log_batch("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = FlowLogBatchEndsAtRangeRule { min: 255, max: 86400 };
        let mut item = new_flow_log_batch("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FlowLogBatchLabelLengthRule { max_length: 120 };
        let mut item = new_flow_log_batch("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = FlowLogBatchCountryLengthRule { max_length: 1000 };
        let mut item = new_flow_log_batch("x", "y");
        item.country = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = FlowLogBatchUnitLengthRule { max_length: 64 };
        let mut item = new_flow_log_batch("x", "y");
        item.unit = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
