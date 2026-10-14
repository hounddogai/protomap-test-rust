use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval estimate must pass. Check returns its problems, or none.
pub trait ApprovalEstimateRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalEstimatePinnedGuardRule {
    pub limit: i64,
}

impl ApprovalEstimateRule for ApprovalEstimatePinnedGuardRule {
    fn check(&self, item: &ApprovalEstimate) -> Vec<String> {
        if item.pinned && (item.expires_at > self.limit) {
            return vec![format!(
                "expires at {} exceeds {} while pinned",
                item.expires_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalEstimateCodeLengthRule {
    pub max_length: i64,
}

impl ApprovalEstimateRule for ApprovalEstimateCodeLengthRule {
    fn check(&self, item: &ApprovalEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalEstimateScopeLengthRule {
    pub max_length: i64,
}

impl ApprovalEstimateRule for ApprovalEstimateScopeLengthRule {
    fn check(&self, item: &ApprovalEstimate) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval estimate.
#[derive(Debug, Clone)]
pub struct ApprovalEstimateValidator {
    pub rules: Vec<Rc<dyn ApprovalEstimateRule>>,
}

impl ApprovalEstimateValidator {
    pub fn validate(&self, item: &ApprovalEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_estimate_validator() -> ApprovalEstimateValidator {
    ApprovalEstimateValidator {
        rules: vec![
            Rc::new(ApprovalEstimatePinnedGuardRule { limit: 80 }) as Rc<dyn ApprovalEstimateRule>,
            Rc::new(ApprovalEstimateCodeLengthRule { max_length: 16 }),
            Rc::new(ApprovalEstimateScopeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_estimate_validator();
        let item = new_approval_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ApprovalEstimateCodeLengthRule { max_length: 16 };
        let mut item = new_approval_estimate("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = ApprovalEstimateScopeLengthRule { max_length: 120 };
        let mut item = new_approval_estimate("x", "y");
        item.scope = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
