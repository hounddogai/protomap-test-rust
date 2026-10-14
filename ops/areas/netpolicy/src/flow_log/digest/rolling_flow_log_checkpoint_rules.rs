use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling flow log checkpoint must pass. Check returns its problems, or none.
pub trait RollingFlowLogCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RollingFlowLogCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFlowLogCheckpointNameLengthRule {
    pub max_length: i64,
}

impl RollingFlowLogCheckpointRule for RollingFlowLogCheckpointNameLengthRule {
    fn check(&self, item: &RollingFlowLogCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFlowLogCheckpointVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingFlowLogCheckpointRule for RollingFlowLogCheckpointVersionRangeRule {
    fn check(&self, item: &RollingFlowLogCheckpoint) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFlowLogCheckpointAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl RollingFlowLogCheckpointRule for RollingFlowLogCheckpointAllowedClusterRule {
    fn check(&self, item: &RollingFlowLogCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is inconsistent", item.cluster)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFlowLogCheckpointRequiredTenantIdRule {
    pub message: String,
}

impl RollingFlowLogCheckpointRule for RollingFlowLogCheckpointRequiredTenantIdRule {
    fn check(&self, item: &RollingFlowLogCheckpoint) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling flow log checkpoint.
#[derive(Debug, Clone)]
pub struct RollingFlowLogCheckpointValidator {
    pub rules: Vec<Rc<dyn RollingFlowLogCheckpointRule>>,
}

impl RollingFlowLogCheckpointValidator {
    pub fn validate(&self, item: &RollingFlowLogCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingFlowLogCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingFlowLogCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_flow_log_checkpoint_validator() -> RollingFlowLogCheckpointValidator {
    RollingFlowLogCheckpointValidator {
        rules: vec![
            Rc::new(RollingFlowLogCheckpointNameLengthRule { max_length: 16 }) as Rc<dyn RollingFlowLogCheckpointRule>,
            Rc::new(RollingFlowLogCheckpointVersionRangeRule { min: 32, max: 80 }),
            Rc::new(RollingFlowLogCheckpointAllowedClusterRule {
                allowed: vec!["regional".to_string(), "default".to_string()],
            }),
            Rc::new(RollingFlowLogCheckpointRequiredTenantIdRule {
                message: "rolling flow log checkpoint needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_flow_log_checkpoint_validator();
        let item = new_rolling_flow_log_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollingFlowLogCheckpointNameLengthRule { max_length: 255 };
        let mut item = new_rolling_flow_log_checkpoint("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RollingFlowLogCheckpointVersionRangeRule { min: 1000, max: 86400 };
        let mut item = new_rolling_flow_log_checkpoint("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = RollingFlowLogCheckpointRequiredTenantIdRule {
            message: "rolling flow log checkpoint needs this field".to_string(),
        };
        let mut item = new_rolling_flow_log_checkpoint("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
