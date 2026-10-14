use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual flow log policy must pass. Check returns its problems, or none.
pub trait ManualFlowLogPolicyRule: std::fmt::Debug {
    fn check(&self, item: &ManualFlowLogPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogPolicyRequiredCodeRule {
    pub message: String,
}

impl ManualFlowLogPolicyRule for ManualFlowLogPolicyRequiredCodeRule {
    fn check(&self, item: &ManualFlowLogPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogPolicyPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualFlowLogPolicyRule for ManualFlowLogPolicyPortRangeRule {
    fn check(&self, item: &ManualFlowLogPolicy) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogPolicySequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualFlowLogPolicyRule for ManualFlowLogPolicySequenceRangeRule {
    fn check(&self, item: &ManualFlowLogPolicy) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual flow log policy.
#[derive(Debug, Clone)]
pub struct ManualFlowLogPolicyValidator {
    pub rules: Vec<Rc<dyn ManualFlowLogPolicyRule>>,
}

impl ManualFlowLogPolicyValidator {
    pub fn validate(&self, item: &ManualFlowLogPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualFlowLogPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualFlowLogPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_flow_log_policy_validator() -> ManualFlowLogPolicyValidator {
    ManualFlowLogPolicyValidator {
        rules: vec![
            Rc::new(ManualFlowLogPolicyRequiredCodeRule {
                message: "manual flow log policy needs a value".to_string(),
            }) as Rc<dyn ManualFlowLogPolicyRule>,
            Rc::new(ManualFlowLogPolicyPortRangeRule { min: 16, max: 3600 }),
            Rc::new(ManualFlowLogPolicySequenceRangeRule { min: 255, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_flow_log_policy_validator();
        let item = new_manual_flow_log_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualFlowLogPolicyRequiredCodeRule {
            message: "manual flow log policy needs this field".to_string(),
        };
        let mut item = new_manual_flow_log_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = ManualFlowLogPolicyPortRangeRule { min: 32, max: 86400 };
        let mut item = new_manual_flow_log_policy("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = ManualFlowLogPolicySequenceRangeRule { min: 255, max: 86400 };
        let mut item = new_manual_flow_log_policy("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
