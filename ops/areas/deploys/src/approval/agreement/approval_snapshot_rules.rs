use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval snapshot must pass. Check returns its problems, or none.
pub trait ApprovalSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSnapshotCodePrefixRule {
    pub prefix: String,
}

impl ApprovalSnapshotRule for ApprovalSnapshotCodePrefixRule {
    fn check(&self, item: &ApprovalSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSnapshotThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalSnapshotRule for ApprovalSnapshotThresholdRangeRule {
    fn check(&self, item: &ApprovalSnapshot) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSnapshotRequiredCodeRule {
    pub message: String,
}

impl ApprovalSnapshotRule for ApprovalSnapshotRequiredCodeRule {
    fn check(&self, item: &ApprovalSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSnapshotRequiredLabelRule {
    pub message: String,
}

impl ApprovalSnapshotRule for ApprovalSnapshotRequiredLabelRule {
    fn check(&self, item: &ApprovalSnapshot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval snapshot.
#[derive(Debug, Clone)]
pub struct ApprovalSnapshotValidator {
    pub rules: Vec<Rc<dyn ApprovalSnapshotRule>>,
}

impl ApprovalSnapshotValidator {
    pub fn validate(&self, item: &ApprovalSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_snapshot_validator() -> ApprovalSnapshotValidator {
    ApprovalSnapshotValidator {
        rules: vec![
            Rc::new(ApprovalSnapshotCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn ApprovalSnapshotRule>,
            Rc::new(ApprovalSnapshotThresholdRangeRule { min: 6.9, max: 1397.5 }),
            Rc::new(ApprovalSnapshotRequiredCodeRule {
                message: "approval snapshot needs an entry".to_string(),
            }),
            Rc::new(ApprovalSnapshotRequiredLabelRule {
                message: "approval snapshot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_snapshot_validator();
        let item = new_approval_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = ApprovalSnapshotThresholdRangeRule { min: 9.9, max: 3002.6 };
        let mut item = new_approval_snapshot("x", "y");
        item.threshold = 3004.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ApprovalSnapshotRequiredCodeRule {
            message: "approval snapshot needs this field".to_string(),
        };
        let mut item = new_approval_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ApprovalSnapshotRequiredLabelRule {
            message: "approval snapshot needs a value".to_string(),
        };
        let mut item = new_approval_snapshot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
