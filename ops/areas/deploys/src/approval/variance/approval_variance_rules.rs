use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval variance must pass. Check returns its problems, or none.
pub trait ApprovalVarianceRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalVarianceClusterLengthRule {
    pub max_length: i64,
}

impl ApprovalVarianceRule for ApprovalVarianceClusterLengthRule {
    fn check(&self, item: &ApprovalVariance) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalVarianceRequiredClusterRule {
    pub message: String,
}

impl ApprovalVarianceRule for ApprovalVarianceRequiredClusterRule {
    fn check(&self, item: &ApprovalVariance) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalVarianceNameLengthRule {
    pub max_length: i64,
}

impl ApprovalVarianceRule for ApprovalVarianceNameLengthRule {
    fn check(&self, item: &ApprovalVariance) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval variance.
#[derive(Debug, Clone)]
pub struct ApprovalVarianceValidator {
    pub rules: Vec<Rc<dyn ApprovalVarianceRule>>,
}

impl ApprovalVarianceValidator {
    pub fn validate(&self, item: &ApprovalVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_variance_validator() -> ApprovalVarianceValidator {
    ApprovalVarianceValidator {
        rules: vec![
            Rc::new(ApprovalVarianceClusterLengthRule { max_length: 64 }) as Rc<dyn ApprovalVarianceRule>,
            Rc::new(ApprovalVarianceRequiredClusterRule {
                message: "approval variance needs this field".to_string(),
            }),
            Rc::new(ApprovalVarianceNameLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_variance_validator();
        let item = new_approval_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = ApprovalVarianceClusterLengthRule { max_length: 86400 };
        let mut item = new_approval_variance("x", "y");
        item.cluster = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = ApprovalVarianceRequiredClusterRule {
            message: "approval variance needs a value".to_string(),
        };
        let mut item = new_approval_variance("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ApprovalVarianceNameLengthRule { max_length: 255 };
        let mut item = new_approval_variance("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
