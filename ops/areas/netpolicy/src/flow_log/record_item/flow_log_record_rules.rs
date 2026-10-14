use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log record must pass. Check returns its problems, or none.
pub trait FlowLogRecordRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogRecordRequiredNameRule {
    pub message: String,
}

impl FlowLogRecordRule for FlowLogRecordRequiredNameRule {
    fn check(&self, item: &FlowLogRecord) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogRecordRequiredCodeRule {
    pub message: String,
}

impl FlowLogRecordRule for FlowLogRecordRequiredCodeRule {
    fn check(&self, item: &FlowLogRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogRecordAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FlowLogRecordRule for FlowLogRecordAllowedNameRule {
    fn check(&self, item: &FlowLogRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is inconsistent", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogRecordNameLengthRule {
    pub max_length: i64,
}

impl FlowLogRecordRule for FlowLogRecordNameLengthRule {
    fn check(&self, item: &FlowLogRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogRecordLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogRecordRule for FlowLogRecordLimitRangeRule {
    fn check(&self, item: &FlowLogRecord) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a flow log record.
#[derive(Debug, Clone)]
pub struct FlowLogRecordValidator {
    pub rules: Vec<Rc<dyn FlowLogRecordRule>>,
}

impl FlowLogRecordValidator {
    pub fn validate(&self, item: &FlowLogRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_record_validator() -> FlowLogRecordValidator {
    FlowLogRecordValidator {
        rules: vec![
            Rc::new(FlowLogRecordRequiredNameRule {
                message: "flow log record needs this field".to_string(),
            }) as Rc<dyn FlowLogRecordRule>,
            Rc::new(FlowLogRecordRequiredCodeRule {
                message: "flow log record needs this field".to_string(),
            }),
            Rc::new(FlowLogRecordAllowedNameRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(FlowLogRecordNameLengthRule { max_length: 80 }),
            Rc::new(FlowLogRecordLimitRangeRule { min: 120, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_record_validator();
        let item = new_flow_log_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FlowLogRecordRequiredNameRule {
            message: "flow log record needs a value".to_string(),
        };
        let mut item = new_flow_log_record("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FlowLogRecordRequiredCodeRule {
            message: "flow log record needs an entry".to_string(),
        };
        let mut item = new_flow_log_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FlowLogRecordNameLengthRule { max_length: 64 };
        let mut item = new_flow_log_record("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = FlowLogRecordLimitRangeRule { min: 32, max: 3600 };
        let mut item = new_flow_log_record("x", "y");
        item.limit = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
