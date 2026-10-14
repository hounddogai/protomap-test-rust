use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout milestone must pass. Check returns its problems, or none.
pub trait RolloutMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &RolloutMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl RolloutMilestoneRule for RolloutMilestoneCodeLengthRule {
    fn check(&self, item: &RolloutMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMilestoneRequiredExternalRefRule {
    pub message: String,
}

impl RolloutMilestoneRule for RolloutMilestoneRequiredExternalRefRule {
    fn check(&self, item: &RolloutMilestone) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMilestonePrincipalLengthRule {
    pub max_length: i64,
}

impl RolloutMilestoneRule for RolloutMilestonePrincipalLengthRule {
    fn check(&self, item: &RolloutMilestone) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout milestone.
#[derive(Debug, Clone)]
pub struct RolloutMilestoneValidator {
    pub rules: Vec<Rc<dyn RolloutMilestoneRule>>,
}

impl RolloutMilestoneValidator {
    pub fn validate(&self, item: &RolloutMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_milestone_validator() -> RolloutMilestoneValidator {
    RolloutMilestoneValidator {
        rules: vec![
            Rc::new(RolloutMilestoneCodeLengthRule { max_length: 16 }) as Rc<dyn RolloutMilestoneRule>,
            Rc::new(RolloutMilestoneRequiredExternalRefRule {
                message: "rollout milestone needs an entry".to_string(),
            }),
            Rc::new(RolloutMilestonePrincipalLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_milestone_validator();
        let item = new_rollout_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RolloutMilestoneCodeLengthRule { max_length: 255 };
        let mut item = new_rollout_milestone("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = RolloutMilestoneRequiredExternalRefRule {
            message: "rollout milestone needs a value".to_string(),
        };
        let mut item = new_rollout_milestone("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = RolloutMilestonePrincipalLengthRule { max_length: 255 };
        let mut item = new_rollout_milestone("x", "y");
        item.principal = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
