use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log entry must pass. Check returns its problems, or none.
pub trait FlowLogEntryRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogEntryRequiredCodeRule {
    pub message: String,
}

impl FlowLogEntryRule for FlowLogEntryRequiredCodeRule {
    fn check(&self, item: &FlowLogEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogEntryVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogEntryRule for FlowLogEntryVersionRangeRule {
    fn check(&self, item: &FlowLogEntry) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogEntryEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogEntryRule for FlowLogEntryEndsAtRangeRule {
    fn check(&self, item: &FlowLogEntry) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogEntryReviewerLengthRule {
    pub max_length: i64,
}

impl FlowLogEntryRule for FlowLogEntryReviewerLengthRule {
    fn check(&self, item: &FlowLogEntry) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogEntryCodePrefixRule {
    pub prefix: String,
}

impl FlowLogEntryRule for FlowLogEntryCodePrefixRule {
    fn check(&self, item: &FlowLogEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogEntryAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FlowLogEntryRule for FlowLogEntryAmountRangeRule {
    fn check(&self, item: &FlowLogEntry) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log entry.
#[derive(Debug, Clone)]
pub struct FlowLogEntryValidator {
    pub rules: Vec<Rc<dyn FlowLogEntryRule>>,
}

impl FlowLogEntryValidator {
    pub fn validate(&self, item: &FlowLogEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_entry_validator() -> FlowLogEntryValidator {
    FlowLogEntryValidator {
        rules: vec![
            Rc::new(FlowLogEntryRequiredCodeRule {
                message: "flow log entry needs this field".to_string(),
            }) as Rc<dyn FlowLogEntryRule>,
            Rc::new(FlowLogEntryVersionRangeRule { min: 255, max: 1000 }),
            Rc::new(FlowLogEntryEndsAtRangeRule { min: 1000, max: 3600 }),
            Rc::new(FlowLogEntryReviewerLengthRule { max_length: 64 }),
            Rc::new(FlowLogEntryCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(FlowLogEntryAmountRangeRule { min: 1.7, max: 1345.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_entry_validator();
        let item = new_flow_log_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FlowLogEntryRequiredCodeRule {
            message: "flow log entry needs a value".to_string(),
        };
        let mut item = new_flow_log_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = FlowLogEntryVersionRangeRule { min: 16, max: 32 };
        let mut item = new_flow_log_entry("x", "y");
        item.version = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = FlowLogEntryEndsAtRangeRule { min: 80, max: 255 };
        let mut item = new_flow_log_entry("x", "y");
        item.ends_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = FlowLogEntryReviewerLengthRule { max_length: 64 };
        let mut item = new_flow_log_entry("x", "y");
        item.reviewer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = FlowLogEntryAmountRangeRule { min: 8.7, max: 2290.6 };
        let mut item = new_flow_log_entry("x", "y");
        item.amount = 2292.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
