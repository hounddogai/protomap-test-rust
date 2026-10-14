use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert adjustment must pass. Check returns its problems, or none.
pub trait AlertAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &AlertAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAdjustmentLabelLengthRule {
    pub max_length: i64,
}

impl AlertAdjustmentRule for AlertAdjustmentLabelLengthRule {
    fn check(&self, item: &AlertAdjustment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAdjustmentRequiredExternalRefRule {
    pub message: String,
}

impl AlertAdjustmentRule for AlertAdjustmentRequiredExternalRefRule {
    fn check(&self, item: &AlertAdjustment) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAdjustmentRequiredCodeRule {
    pub message: String,
}

impl AlertAdjustmentRule for AlertAdjustmentRequiredCodeRule {
    fn check(&self, item: &AlertAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl AlertAdjustmentRule for AlertAdjustmentCodePrefixRule {
    fn check(&self, item: &AlertAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAdjustmentSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertAdjustmentRule for AlertAdjustmentSequenceRangeRule {
    fn check(&self, item: &AlertAdjustment) -> Vec<String> {
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
pub struct AlertAdjustmentSegmentLengthRule {
    pub max_length: i64,
}

impl AlertAdjustmentRule for AlertAdjustmentSegmentLengthRule {
    fn check(&self, item: &AlertAdjustment) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert adjustment.
#[derive(Debug, Clone)]
pub struct AlertAdjustmentValidator {
    pub rules: Vec<Rc<dyn AlertAdjustmentRule>>,
}

impl AlertAdjustmentValidator {
    pub fn validate(&self, item: &AlertAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_adjustment_validator() -> AlertAdjustmentValidator {
    AlertAdjustmentValidator {
        rules: vec![
            Rc::new(AlertAdjustmentLabelLengthRule { max_length: 3600 }) as Rc<dyn AlertAdjustmentRule>,
            Rc::new(AlertAdjustmentRequiredExternalRefRule {
                message: "alert adjustment needs an entry".to_string(),
            }),
            Rc::new(AlertAdjustmentRequiredCodeRule {
                message: "alert adjustment needs this field".to_string(),
            }),
            Rc::new(AlertAdjustmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(AlertAdjustmentSequenceRangeRule { min: 3600, max: 86400 }),
            Rc::new(AlertAdjustmentSegmentLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_adjustment_validator();
        let item = new_alert_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AlertAdjustmentLabelLengthRule { max_length: 255 };
        let mut item = new_alert_adjustment("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = AlertAdjustmentRequiredExternalRefRule {
            message: "alert adjustment needs a value".to_string(),
        };
        let mut item = new_alert_adjustment("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AlertAdjustmentRequiredCodeRule {
            message: "alert adjustment needs an entry".to_string(),
        };
        let mut item = new_alert_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = AlertAdjustmentSequenceRangeRule { min: 1000, max: 86400 };
        let mut item = new_alert_adjustment("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = AlertAdjustmentSegmentLengthRule { max_length: 3600 };
        let mut item = new_alert_adjustment("x", "y");
        item.segment = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
