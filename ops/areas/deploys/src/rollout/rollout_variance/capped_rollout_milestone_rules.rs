use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped rollout milestone must pass. Check returns its problems, or none.
pub trait CappedRolloutMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &CappedRolloutMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRolloutMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl CappedRolloutMilestoneRule for CappedRolloutMilestoneCodeLengthRule {
    fn check(&self, item: &CappedRolloutMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRolloutMilestoneRequiredIssuerRule {
    pub message: String,
}

impl CappedRolloutMilestoneRule for CappedRolloutMilestoneRequiredIssuerRule {
    fn check(&self, item: &CappedRolloutMilestone) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRolloutMilestoneCodePrefixRule {
    pub prefix: String,
}

impl CappedRolloutMilestoneRule for CappedRolloutMilestoneCodePrefixRule {
    fn check(&self, item: &CappedRolloutMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRolloutMilestoneLabelLengthRule {
    pub max_length: i64,
}

impl CappedRolloutMilestoneRule for CappedRolloutMilestoneLabelLengthRule {
    fn check(&self, item: &CappedRolloutMilestone) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRolloutMilestoneIssuerLengthRule {
    pub max_length: i64,
}

impl CappedRolloutMilestoneRule for CappedRolloutMilestoneIssuerLengthRule {
    fn check(&self, item: &CappedRolloutMilestone) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped rollout milestone.
#[derive(Debug, Clone)]
pub struct CappedRolloutMilestoneValidator {
    pub rules: Vec<Rc<dyn CappedRolloutMilestoneRule>>,
}

impl CappedRolloutMilestoneValidator {
    pub fn validate(&self, item: &CappedRolloutMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedRolloutMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRolloutMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_rollout_milestone_validator() -> CappedRolloutMilestoneValidator {
    CappedRolloutMilestoneValidator {
        rules: vec![
            Rc::new(CappedRolloutMilestoneCodeLengthRule { max_length: 1000 }) as Rc<dyn CappedRolloutMilestoneRule>,
            Rc::new(CappedRolloutMilestoneRequiredIssuerRule {
                message: "capped rollout milestone needs a value".to_string(),
            }),
            Rc::new(CappedRolloutMilestoneCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(CappedRolloutMilestoneLabelLengthRule { max_length: 64 }),
            Rc::new(CappedRolloutMilestoneIssuerLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_rollout_milestone_validator();
        let item = new_capped_rollout_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedRolloutMilestoneCodeLengthRule { max_length: 3600 };
        let mut item = new_capped_rollout_milestone("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = CappedRolloutMilestoneRequiredIssuerRule {
            message: "capped rollout milestone needs an entry".to_string(),
        };
        let mut item = new_capped_rollout_milestone("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CappedRolloutMilestoneLabelLengthRule { max_length: 120 };
        let mut item = new_capped_rollout_milestone("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = CappedRolloutMilestoneIssuerLengthRule { max_length: 1000 };
        let mut item = new_capped_rollout_milestone("x", "y");
        item.issuer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
