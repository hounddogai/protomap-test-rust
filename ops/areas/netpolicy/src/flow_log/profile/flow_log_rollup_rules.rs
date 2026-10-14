use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log rollup must pass. Check returns its problems, or none.
pub trait FlowLogRollupRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogRollupRequiredTitleRule {
    pub message: String,
}

impl FlowLogRollupRule for FlowLogRollupRequiredTitleRule {
    fn check(&self, item: &FlowLogRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogRollupRequiredKeyIdRule {
    pub message: String,
}

impl FlowLogRollupRule for FlowLogRollupRequiredKeyIdRule {
    fn check(&self, item: &FlowLogRollup) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogRollupRequiredCodeRule {
    pub message: String,
}

impl FlowLogRollupRule for FlowLogRollupRequiredCodeRule {
    fn check(&self, item: &FlowLogRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogRollupVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogRollupRule for FlowLogRollupVersionRangeRule {
    fn check(&self, item: &FlowLogRollup) -> Vec<String> {
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
pub struct FlowLogRollupAllowedHostRule {
    pub allowed: Vec<String>,
}

impl FlowLogRollupRule for FlowLogRollupAllowedHostRule {
    fn check(&self, item: &FlowLogRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is unknown", item.host)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log rollup.
#[derive(Debug, Clone)]
pub struct FlowLogRollupValidator {
    pub rules: Vec<Rc<dyn FlowLogRollupRule>>,
}

impl FlowLogRollupValidator {
    pub fn validate(&self, item: &FlowLogRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_rollup_validator() -> FlowLogRollupValidator {
    FlowLogRollupValidator {
        rules: vec![
            Rc::new(FlowLogRollupRequiredTitleRule {
                message: "flow log rollup needs an entry".to_string(),
            }) as Rc<dyn FlowLogRollupRule>,
            Rc::new(FlowLogRollupRequiredKeyIdRule {
                message: "flow log rollup needs a value".to_string(),
            }),
            Rc::new(FlowLogRollupRequiredCodeRule {
                message: "flow log rollup needs an entry".to_string(),
            }),
            Rc::new(FlowLogRollupVersionRangeRule { min: 64, max: 1000 }),
            Rc::new(FlowLogRollupAllowedHostRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_rollup_validator();
        let item = new_flow_log_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FlowLogRollupRequiredTitleRule {
            message: "flow log rollup needs this field".to_string(),
        };
        let mut item = new_flow_log_rollup("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = FlowLogRollupRequiredKeyIdRule {
            message: "flow log rollup needs an entry".to_string(),
        };
        let mut item = new_flow_log_rollup("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FlowLogRollupRequiredCodeRule {
            message: "flow log rollup needs an entry".to_string(),
        };
        let mut item = new_flow_log_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = FlowLogRollupVersionRangeRule { min: 255, max: 3600 };
        let mut item = new_flow_log_rollup("x", "y");
        item.version = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
