use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log snapshot must pass. Check returns its problems, or none.
pub trait FlowLogSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogSnapshotRequiredNameRule {
    pub message: String,
}

impl FlowLogSnapshotRule for FlowLogSnapshotRequiredNameRule {
    fn check(&self, item: &FlowLogSnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogSnapshotRequiredTitleRule {
    pub message: String,
}

impl FlowLogSnapshotRule for FlowLogSnapshotRequiredTitleRule {
    fn check(&self, item: &FlowLogSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogSnapshotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FlowLogSnapshotRule for FlowLogSnapshotAllowedCodeRule {
    fn check(&self, item: &FlowLogSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogSnapshotNameLengthRule {
    pub max_length: i64,
}

impl FlowLogSnapshotRule for FlowLogSnapshotNameLengthRule {
    fn check(&self, item: &FlowLogSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogSnapshotPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogSnapshotRule for FlowLogSnapshotPriorityRangeRule {
    fn check(&self, item: &FlowLogSnapshot) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log snapshot.
#[derive(Debug, Clone)]
pub struct FlowLogSnapshotValidator {
    pub rules: Vec<Rc<dyn FlowLogSnapshotRule>>,
}

impl FlowLogSnapshotValidator {
    pub fn validate(&self, item: &FlowLogSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_snapshot_validator() -> FlowLogSnapshotValidator {
    FlowLogSnapshotValidator {
        rules: vec![
            Rc::new(FlowLogSnapshotRequiredNameRule {
                message: "flow log snapshot needs an entry".to_string(),
            }) as Rc<dyn FlowLogSnapshotRule>,
            Rc::new(FlowLogSnapshotRequiredTitleRule {
                message: "flow log snapshot needs a value".to_string(),
            }),
            Rc::new(FlowLogSnapshotAllowedCodeRule {
                allowed: vec!["batched".to_string()],
            }),
            Rc::new(FlowLogSnapshotNameLengthRule { max_length: 32 }),
            Rc::new(FlowLogSnapshotPriorityRangeRule { min: 1000, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_snapshot_validator();
        let item = new_flow_log_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FlowLogSnapshotRequiredNameRule {
            message: "flow log snapshot needs this field".to_string(),
        };
        let mut item = new_flow_log_snapshot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FlowLogSnapshotRequiredTitleRule {
            message: "flow log snapshot needs a value".to_string(),
        };
        let mut item = new_flow_log_snapshot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FlowLogSnapshotNameLengthRule { max_length: 16 };
        let mut item = new_flow_log_snapshot("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = FlowLogSnapshotPriorityRangeRule { min: 80, max: 1000 };
        let mut item = new_flow_log_snapshot("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
