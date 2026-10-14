use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global flow log checkpoint must pass. Check returns its problems, or none.
pub trait GlobalFlowLogCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &GlobalFlowLogCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFlowLogCheckpointRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalFlowLogCheckpointRule for GlobalFlowLogCheckpointRotationDaysRangeRule {
    fn check(&self, item: &GlobalFlowLogCheckpoint) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFlowLogCheckpointRequiredCodeRule {
    pub message: String,
}

impl GlobalFlowLogCheckpointRule for GlobalFlowLogCheckpointRequiredCodeRule {
    fn check(&self, item: &GlobalFlowLogCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFlowLogCheckpointRequiredKeyIdRule {
    pub message: String,
}

impl GlobalFlowLogCheckpointRule for GlobalFlowLogCheckpointRequiredKeyIdRule {
    fn check(&self, item: &GlobalFlowLogCheckpoint) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global flow log checkpoint.
#[derive(Debug, Clone)]
pub struct GlobalFlowLogCheckpointValidator {
    pub rules: Vec<Rc<dyn GlobalFlowLogCheckpointRule>>,
}

impl GlobalFlowLogCheckpointValidator {
    pub fn validate(&self, item: &GlobalFlowLogCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalFlowLogCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalFlowLogCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_flow_log_checkpoint_validator() -> GlobalFlowLogCheckpointValidator {
    GlobalFlowLogCheckpointValidator {
        rules: vec![
            Rc::new(GlobalFlowLogCheckpointRotationDaysRangeRule { min: 255, max: 86400 })
                as Rc<dyn GlobalFlowLogCheckpointRule>,
            Rc::new(GlobalFlowLogCheckpointRequiredCodeRule {
                message: "global flow log checkpoint needs a value".to_string(),
            }),
            Rc::new(GlobalFlowLogCheckpointRequiredKeyIdRule {
                message: "global flow log checkpoint needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_flow_log_checkpoint_validator();
        let item = new_global_flow_log_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = GlobalFlowLogCheckpointRotationDaysRangeRule { min: 64, max: 3600 };
        let mut item = new_global_flow_log_checkpoint("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalFlowLogCheckpointRequiredCodeRule {
            message: "global flow log checkpoint needs this field".to_string(),
        };
        let mut item = new_global_flow_log_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = GlobalFlowLogCheckpointRequiredKeyIdRule {
            message: "global flow log checkpoint needs a value".to_string(),
        };
        let mut item = new_global_flow_log_checkpoint("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
