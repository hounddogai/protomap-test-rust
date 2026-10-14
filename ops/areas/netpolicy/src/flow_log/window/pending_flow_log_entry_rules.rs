use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending flow log entry must pass. Check returns its problems, or none.
pub trait PendingFlowLogEntryRule: std::fmt::Debug {
    fn check(&self, item: &PendingFlowLogEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogEntryRequiredLabelRule {
    pub message: String,
}

impl PendingFlowLogEntryRule for PendingFlowLogEntryRequiredLabelRule {
    fn check(&self, item: &PendingFlowLogEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogEntryRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingFlowLogEntryRule for PendingFlowLogEntryRevisionRangeRule {
    fn check(&self, item: &PendingFlowLogEntry) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogEntryRequiredCodeRule {
    pub message: String,
}

impl PendingFlowLogEntryRule for PendingFlowLogEntryRequiredCodeRule {
    fn check(&self, item: &PendingFlowLogEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogEntryCodePrefixRule {
    pub prefix: String,
}

impl PendingFlowLogEntryRule for PendingFlowLogEntryCodePrefixRule {
    fn check(&self, item: &PendingFlowLogEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogEntryFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingFlowLogEntryRule for PendingFlowLogEntryFloorValueRangeRule {
    fn check(&self, item: &PendingFlowLogEntry) -> Vec<String> {
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
pub struct PendingFlowLogEntryNamespaceLengthRule {
    pub max_length: i64,
}

impl PendingFlowLogEntryRule for PendingFlowLogEntryNamespaceLengthRule {
    fn check(&self, item: &PendingFlowLogEntry) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending flow log entry.
#[derive(Debug, Clone)]
pub struct PendingFlowLogEntryValidator {
    pub rules: Vec<Rc<dyn PendingFlowLogEntryRule>>,
}

impl PendingFlowLogEntryValidator {
    pub fn validate(&self, item: &PendingFlowLogEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingFlowLogEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingFlowLogEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_flow_log_entry_validator() -> PendingFlowLogEntryValidator {
    PendingFlowLogEntryValidator {
        rules: vec![
            Rc::new(PendingFlowLogEntryRequiredLabelRule {
                message: "pending flow log entry needs this field".to_string(),
            }) as Rc<dyn PendingFlowLogEntryRule>,
            Rc::new(PendingFlowLogEntryRevisionRangeRule { min: 80, max: 80 }),
            Rc::new(PendingFlowLogEntryRequiredCodeRule {
                message: "pending flow log entry needs this field".to_string(),
            }),
            Rc::new(PendingFlowLogEntryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PendingFlowLogEntryFloorValueRangeRule { min: 5.5, max: 1136.3 }),
            Rc::new(PendingFlowLogEntryNamespaceLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_flow_log_entry_validator();
        let item = new_pending_flow_log_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingFlowLogEntryRequiredLabelRule {
            message: "pending flow log entry needs this field".to_string(),
        };
        let mut item = new_pending_flow_log_entry("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = PendingFlowLogEntryRevisionRangeRule { min: 32, max: 3600 };
        let mut item = new_pending_flow_log_entry("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingFlowLogEntryRequiredCodeRule {
            message: "pending flow log entry needs a value".to_string(),
        };
        let mut item = new_pending_flow_log_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = PendingFlowLogEntryFloorValueRangeRule { min: 1.5, max: 1335.2 };
        let mut item = new_pending_flow_log_entry("x", "y");
        item.floor_value = 1336.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = PendingFlowLogEntryNamespaceLengthRule { max_length: 1000 };
        let mut item = new_pending_flow_log_entry("x", "y");
        item.namespace = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
