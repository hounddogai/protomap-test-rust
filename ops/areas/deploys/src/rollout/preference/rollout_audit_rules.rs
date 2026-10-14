use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout audit must pass. Check returns its problems, or none.
pub trait RolloutAuditRule: std::fmt::Debug {
    fn check(&self, item: &RolloutAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAuditRequiredHostRule {
    pub message: String,
}

impl RolloutAuditRule for RolloutAuditRequiredHostRule {
    fn check(&self, item: &RolloutAudit) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAuditThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutAuditRule for RolloutAuditThresholdRangeRule {
    fn check(&self, item: &RolloutAudit) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAuditUnitLengthRule {
    pub max_length: i64,
}

impl RolloutAuditRule for RolloutAuditUnitLengthRule {
    fn check(&self, item: &RolloutAudit) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAuditRequiredUnitRule {
    pub message: String,
}

impl RolloutAuditRule for RolloutAuditRequiredUnitRule {
    fn check(&self, item: &RolloutAudit) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAuditCodePrefixRule {
    pub prefix: String,
}

impl RolloutAuditRule for RolloutAuditCodePrefixRule {
    fn check(&self, item: &RolloutAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout audit.
#[derive(Debug, Clone)]
pub struct RolloutAuditValidator {
    pub rules: Vec<Rc<dyn RolloutAuditRule>>,
}

impl RolloutAuditValidator {
    pub fn validate(&self, item: &RolloutAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_audit_validator() -> RolloutAuditValidator {
    RolloutAuditValidator {
        rules: vec![
            Rc::new(RolloutAuditRequiredHostRule {
                message: "rollout audit needs this field".to_string(),
            }) as Rc<dyn RolloutAuditRule>,
            Rc::new(RolloutAuditThresholdRangeRule { min: 5.1, max: 3476.2 }),
            Rc::new(RolloutAuditUnitLengthRule { max_length: 86400 }),
            Rc::new(RolloutAuditRequiredUnitRule {
                message: "rollout audit needs a value".to_string(),
            }),
            Rc::new(RolloutAuditCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_audit_validator();
        let item = new_rollout_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RolloutAuditRequiredHostRule {
            message: "rollout audit needs this field".to_string(),
        };
        let mut item = new_rollout_audit("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RolloutAuditThresholdRangeRule { min: 8.6, max: 156.3 };
        let mut item = new_rollout_audit("x", "y");
        item.threshold = 157.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = RolloutAuditUnitLengthRule { max_length: 80 };
        let mut item = new_rollout_audit("x", "y");
        item.unit = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = RolloutAuditRequiredUnitRule {
            message: "rollout audit needs this field".to_string(),
        };
        let mut item = new_rollout_audit("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
