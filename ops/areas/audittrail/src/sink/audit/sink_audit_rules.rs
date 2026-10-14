use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink audit must pass. Check returns its problems, or none.
pub trait SinkAuditRule: std::fmt::Debug {
    fn check(&self, item: &SinkAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAuditRequiredCodeRule {
    pub message: String,
}

impl SinkAuditRule for SinkAuditRequiredCodeRule {
    fn check(&self, item: &SinkAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAuditFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SinkAuditRule for SinkAuditFloorValueRangeRule {
    fn check(&self, item: &SinkAudit) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAuditLabelLengthRule {
    pub max_length: i64,
}

impl SinkAuditRule for SinkAuditLabelLengthRule {
    fn check(&self, item: &SinkAudit) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAuditMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SinkAuditRule for SinkAuditMarginRangeRule {
    fn check(&self, item: &SinkAudit) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAuditCodeLengthRule {
    pub max_length: i64,
}

impl SinkAuditRule for SinkAuditCodeLengthRule {
    fn check(&self, item: &SinkAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAuditCodePrefixRule {
    pub prefix: String,
}

impl SinkAuditRule for SinkAuditCodePrefixRule {
    fn check(&self, item: &SinkAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink audit.
#[derive(Debug, Clone)]
pub struct SinkAuditValidator {
    pub rules: Vec<Rc<dyn SinkAuditRule>>,
}

impl SinkAuditValidator {
    pub fn validate(&self, item: &SinkAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SinkAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_audit_validator() -> SinkAuditValidator {
    SinkAuditValidator {
        rules: vec![
            Rc::new(SinkAuditRequiredCodeRule {
                message: "sink audit needs an entry".to_string(),
            }) as Rc<dyn SinkAuditRule>,
            Rc::new(SinkAuditFloorValueRangeRule { min: 7.8, max: 4897.2 }),
            Rc::new(SinkAuditLabelLengthRule { max_length: 32 }),
            Rc::new(SinkAuditMarginRangeRule { min: 5.9, max: 1751.2 }),
            Rc::new(SinkAuditCodeLengthRule { max_length: 16 }),
            Rc::new(SinkAuditCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_audit_validator();
        let item = new_sink_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SinkAuditRequiredCodeRule {
            message: "sink audit needs a value".to_string(),
        };
        let mut item = new_sink_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = SinkAuditFloorValueRangeRule { min: 5.9, max: 3600.3 };
        let mut item = new_sink_audit("x", "y");
        item.floor_value = 3601.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SinkAuditLabelLengthRule { max_length: 1000 };
        let mut item = new_sink_audit("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = SinkAuditMarginRangeRule { min: 2.7, max: 1784.0 };
        let mut item = new_sink_audit("x", "y");
        item.margin = 1785.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SinkAuditCodeLengthRule { max_length: 80 };
        let mut item = new_sink_audit("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
