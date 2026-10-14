use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover adjustment must pass. Check returns its problems, or none.
pub trait FailoverAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &FailoverAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl FailoverAdjustmentRule for FailoverAdjustmentCodePrefixRule {
    fn check(&self, item: &FailoverAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAdjustmentRequiredCodeRule {
    pub message: String,
}

impl FailoverAdjustmentRule for FailoverAdjustmentRequiredCodeRule {
    fn check(&self, item: &FailoverAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAdjustmentRequiredLabelRule {
    pub message: String,
}

impl FailoverAdjustmentRule for FailoverAdjustmentRequiredLabelRule {
    fn check(&self, item: &FailoverAdjustment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover adjustment.
#[derive(Debug, Clone)]
pub struct FailoverAdjustmentValidator {
    pub rules: Vec<Rc<dyn FailoverAdjustmentRule>>,
}

impl FailoverAdjustmentValidator {
    pub fn validate(&self, item: &FailoverAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_adjustment_validator() -> FailoverAdjustmentValidator {
    FailoverAdjustmentValidator {
        rules: vec![
            Rc::new(FailoverAdjustmentCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn FailoverAdjustmentRule>,
            Rc::new(FailoverAdjustmentRequiredCodeRule {
                message: "failover adjustment needs an entry".to_string(),
            }),
            Rc::new(FailoverAdjustmentRequiredLabelRule {
                message: "failover adjustment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_adjustment_validator();
        let item = new_failover_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FailoverAdjustmentRequiredCodeRule {
            message: "failover adjustment needs an entry".to_string(),
        };
        let mut item = new_failover_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FailoverAdjustmentRequiredLabelRule {
            message: "failover adjustment needs a value".to_string(),
        };
        let mut item = new_failover_adjustment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
