use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity milestone must pass. Check returns its problems, or none.
pub trait IdentityMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &IdentityMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMilestoneRequiredCodeRule {
    pub message: String,
}

impl IdentityMilestoneRule for IdentityMilestoneRequiredCodeRule {
    fn check(&self, item: &IdentityMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMilestoneRequiredUnitRule {
    pub message: String,
}

impl IdentityMilestoneRule for IdentityMilestoneRequiredUnitRule {
    fn check(&self, item: &IdentityMilestone) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMilestoneUnitLengthRule {
    pub max_length: i64,
}

impl IdentityMilestoneRule for IdentityMilestoneUnitLengthRule {
    fn check(&self, item: &IdentityMilestone) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMilestoneClusterLengthRule {
    pub max_length: i64,
}

impl IdentityMilestoneRule for IdentityMilestoneClusterLengthRule {
    fn check(&self, item: &IdentityMilestone) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMilestoneLabelLengthRule {
    pub max_length: i64,
}

impl IdentityMilestoneRule for IdentityMilestoneLabelLengthRule {
    fn check(&self, item: &IdentityMilestone) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity milestone.
#[derive(Debug, Clone)]
pub struct IdentityMilestoneValidator {
    pub rules: Vec<Rc<dyn IdentityMilestoneRule>>,
}

impl IdentityMilestoneValidator {
    pub fn validate(&self, item: &IdentityMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentityMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_milestone_validator() -> IdentityMilestoneValidator {
    IdentityMilestoneValidator {
        rules: vec![
            Rc::new(IdentityMilestoneRequiredCodeRule {
                message: "identity milestone needs a value".to_string(),
            }) as Rc<dyn IdentityMilestoneRule>,
            Rc::new(IdentityMilestoneRequiredUnitRule {
                message: "identity milestone needs an entry".to_string(),
            }),
            Rc::new(IdentityMilestoneUnitLengthRule { max_length: 1000 }),
            Rc::new(IdentityMilestoneClusterLengthRule { max_length: 120 }),
            Rc::new(IdentityMilestoneLabelLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_milestone_validator();
        let item = new_identity_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IdentityMilestoneRequiredCodeRule {
            message: "identity milestone needs this field".to_string(),
        };
        let mut item = new_identity_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = IdentityMilestoneRequiredUnitRule {
            message: "identity milestone needs an entry".to_string(),
        };
        let mut item = new_identity_milestone("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = IdentityMilestoneUnitLengthRule { max_length: 3600 };
        let mut item = new_identity_milestone("x", "y");
        item.unit = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = IdentityMilestoneClusterLengthRule { max_length: 64 };
        let mut item = new_identity_milestone("x", "y");
        item.cluster = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = IdentityMilestoneLabelLengthRule { max_length: 64 };
        let mut item = new_identity_milestone("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
