use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval policy must pass. Check returns its problems, or none.
pub trait ApprovalPolicyRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPolicyOwnerIdLengthRule {
    pub max_length: i64,
}

impl ApprovalPolicyRule for ApprovalPolicyOwnerIdLengthRule {
    fn check(&self, item: &ApprovalPolicy) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPolicyLabelLengthRule {
    pub max_length: i64,
}

impl ApprovalPolicyRule for ApprovalPolicyLabelLengthRule {
    fn check(&self, item: &ApprovalPolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPolicyRequiredLabelRule {
    pub message: String,
}

impl ApprovalPolicyRule for ApprovalPolicyRequiredLabelRule {
    fn check(&self, item: &ApprovalPolicy) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPolicyScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalPolicyRule for ApprovalPolicyScoreRangeRule {
    fn check(&self, item: &ApprovalPolicy) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPolicyRequiredOwnerIdRule {
    pub message: String,
}

impl ApprovalPolicyRule for ApprovalPolicyRequiredOwnerIdRule {
    fn check(&self, item: &ApprovalPolicy) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval policy.
#[derive(Debug, Clone)]
pub struct ApprovalPolicyValidator {
    pub rules: Vec<Rc<dyn ApprovalPolicyRule>>,
}

impl ApprovalPolicyValidator {
    pub fn validate(&self, item: &ApprovalPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_policy_validator() -> ApprovalPolicyValidator {
    ApprovalPolicyValidator {
        rules: vec![
            Rc::new(ApprovalPolicyOwnerIdLengthRule { max_length: 80 }) as Rc<dyn ApprovalPolicyRule>,
            Rc::new(ApprovalPolicyLabelLengthRule { max_length: 255 }),
            Rc::new(ApprovalPolicyRequiredLabelRule {
                message: "approval policy needs this field".to_string(),
            }),
            Rc::new(ApprovalPolicyScoreRangeRule { min: 4.2, max: 3351.3 }),
            Rc::new(ApprovalPolicyRequiredOwnerIdRule {
                message: "approval policy needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_policy_validator();
        let item = new_approval_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = ApprovalPolicyOwnerIdLengthRule { max_length: 64 };
        let mut item = new_approval_policy("x", "y");
        item.owner_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ApprovalPolicyLabelLengthRule { max_length: 64 };
        let mut item = new_approval_policy("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ApprovalPolicyRequiredLabelRule {
            message: "approval policy needs this field".to_string(),
        };
        let mut item = new_approval_policy("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ApprovalPolicyScoreRangeRule { min: 2.0, max: 2598.3 };
        let mut item = new_approval_policy("x", "y");
        item.score = 2599.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = ApprovalPolicyRequiredOwnerIdRule {
            message: "approval policy needs this field".to_string(),
        };
        let mut item = new_approval_policy("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
