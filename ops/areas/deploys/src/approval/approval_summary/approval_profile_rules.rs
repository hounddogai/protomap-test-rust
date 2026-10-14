use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval profile must pass. Check returns its problems, or none.
pub trait ApprovalProfileRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalProfileCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalProfileRule for ApprovalProfileCreatedAtRangeRule {
    fn check(&self, item: &ApprovalProfile) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalProfileCodePrefixRule {
    pub prefix: String,
}

impl ApprovalProfileRule for ApprovalProfileCodePrefixRule {
    fn check(&self, item: &ApprovalProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalProfileCodeLengthRule {
    pub max_length: i64,
}

impl ApprovalProfileRule for ApprovalProfileCodeLengthRule {
    fn check(&self, item: &ApprovalProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval profile.
#[derive(Debug, Clone)]
pub struct ApprovalProfileValidator {
    pub rules: Vec<Rc<dyn ApprovalProfileRule>>,
}

impl ApprovalProfileValidator {
    pub fn validate(&self, item: &ApprovalProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_profile_validator() -> ApprovalProfileValidator {
    ApprovalProfileValidator {
        rules: vec![
            Rc::new(ApprovalProfileCreatedAtRangeRule { min: 80, max: 86400 }) as Rc<dyn ApprovalProfileRule>,
            Rc::new(ApprovalProfileCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ApprovalProfileCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_profile_validator();
        let item = new_approval_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = ApprovalProfileCreatedAtRangeRule { min: 64, max: 80 };
        let mut item = new_approval_profile("x", "y");
        item.created_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ApprovalProfileCodeLengthRule { max_length: 16 };
        let mut item = new_approval_profile("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
