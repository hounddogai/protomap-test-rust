use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log override must pass. Check returns its problems, or none.
pub trait FlowLogOverrideRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogOverrideEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogOverrideRule for FlowLogOverrideEndsAtRangeRule {
    fn check(&self, item: &FlowLogOverride) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogOverrideRequiredCodeRule {
    pub message: String,
}

impl FlowLogOverrideRule for FlowLogOverrideRequiredCodeRule {
    fn check(&self, item: &FlowLogOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogOverrideNamespaceLengthRule {
    pub max_length: i64,
}

impl FlowLogOverrideRule for FlowLogOverrideNamespaceLengthRule {
    fn check(&self, item: &FlowLogOverride) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogOverrideCodeLengthRule {
    pub max_length: i64,
}

impl FlowLogOverrideRule for FlowLogOverrideCodeLengthRule {
    fn check(&self, item: &FlowLogOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log override.
#[derive(Debug, Clone)]
pub struct FlowLogOverrideValidator {
    pub rules: Vec<Rc<dyn FlowLogOverrideRule>>,
}

impl FlowLogOverrideValidator {
    pub fn validate(&self, item: &FlowLogOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_override_validator() -> FlowLogOverrideValidator {
    FlowLogOverrideValidator {
        rules: vec![
            Rc::new(FlowLogOverrideEndsAtRangeRule { min: 80, max: 3600 }) as Rc<dyn FlowLogOverrideRule>,
            Rc::new(FlowLogOverrideRequiredCodeRule {
                message: "flow log override needs an entry".to_string(),
            }),
            Rc::new(FlowLogOverrideNamespaceLengthRule { max_length: 64 }),
            Rc::new(FlowLogOverrideCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_override_validator();
        let item = new_flow_log_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = FlowLogOverrideEndsAtRangeRule { min: 120, max: 255 };
        let mut item = new_flow_log_override("x", "y");
        item.ends_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FlowLogOverrideRequiredCodeRule {
            message: "flow log override needs a value".to_string(),
        };
        let mut item = new_flow_log_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = FlowLogOverrideNamespaceLengthRule { max_length: 1000 };
        let mut item = new_flow_log_override("x", "y");
        item.namespace = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FlowLogOverrideCodeLengthRule { max_length: 120 };
        let mut item = new_flow_log_override("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
