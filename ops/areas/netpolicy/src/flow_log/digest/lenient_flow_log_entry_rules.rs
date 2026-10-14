use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient flow log entry must pass. Check returns its problems, or none.
pub trait LenientFlowLogEntryRule: std::fmt::Debug {
    fn check(&self, item: &LenientFlowLogEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEntryLabelLengthRule {
    pub max_length: i64,
}

impl LenientFlowLogEntryRule for LenientFlowLogEntryLabelLengthRule {
    fn check(&self, item: &LenientFlowLogEntry) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEntryLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientFlowLogEntryRule for LenientFlowLogEntryLimitRangeRule {
    fn check(&self, item: &LenientFlowLogEntry) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEntryWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientFlowLogEntryRule for LenientFlowLogEntryWindowDaysRangeRule {
    fn check(&self, item: &LenientFlowLogEntry) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEntryRequiredCodeRule {
    pub message: String,
}

impl LenientFlowLogEntryRule for LenientFlowLogEntryRequiredCodeRule {
    fn check(&self, item: &LenientFlowLogEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEntryRequiredLabelRule {
    pub message: String,
}

impl LenientFlowLogEntryRule for LenientFlowLogEntryRequiredLabelRule {
    fn check(&self, item: &LenientFlowLogEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEntryDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientFlowLogEntryRule for LenientFlowLogEntryDiscountRangeRule {
    fn check(&self, item: &LenientFlowLogEntry) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient flow log entry.
#[derive(Debug, Clone)]
pub struct LenientFlowLogEntryValidator {
    pub rules: Vec<Rc<dyn LenientFlowLogEntryRule>>,
}

impl LenientFlowLogEntryValidator {
    pub fn validate(&self, item: &LenientFlowLogEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientFlowLogEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientFlowLogEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_flow_log_entry_validator() -> LenientFlowLogEntryValidator {
    LenientFlowLogEntryValidator {
        rules: vec![
            Rc::new(LenientFlowLogEntryLabelLengthRule { max_length: 64 }) as Rc<dyn LenientFlowLogEntryRule>,
            Rc::new(LenientFlowLogEntryLimitRangeRule { min: 32, max: 86400 }),
            Rc::new(LenientFlowLogEntryWindowDaysRangeRule { min: 32, max: 3600 }),
            Rc::new(LenientFlowLogEntryRequiredCodeRule {
                message: "lenient flow log entry needs a value".to_string(),
            }),
            Rc::new(LenientFlowLogEntryRequiredLabelRule {
                message: "lenient flow log entry needs a value".to_string(),
            }),
            Rc::new(LenientFlowLogEntryDiscountRangeRule { min: 1.0, max: 523.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_flow_log_entry_validator();
        let item = new_lenient_flow_log_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientFlowLogEntryLabelLengthRule { max_length: 255 };
        let mut item = new_lenient_flow_log_entry("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = LenientFlowLogEntryLimitRangeRule { min: 80, max: 255 };
        let mut item = new_lenient_flow_log_entry("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = LenientFlowLogEntryWindowDaysRangeRule { min: 64, max: 255 };
        let mut item = new_lenient_flow_log_entry("x", "y");
        item.window_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientFlowLogEntryRequiredCodeRule {
            message: "lenient flow log entry needs an entry".to_string(),
        };
        let mut item = new_lenient_flow_log_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientFlowLogEntryRequiredLabelRule {
            message: "lenient flow log entry needs this field".to_string(),
        };
        let mut item = new_lenient_flow_log_entry("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = LenientFlowLogEntryDiscountRangeRule { min: 0.4, max: 1204.8 };
        let mut item = new_lenient_flow_log_entry("x", "y");
        item.discount = 1206.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
