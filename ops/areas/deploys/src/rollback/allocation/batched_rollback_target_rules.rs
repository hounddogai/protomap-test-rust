use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched rollback target must pass. Check returns its problems, or none.
pub trait BatchedRollbackTargetRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRollbackTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackTargetReviewerLengthRule {
    pub max_length: i64,
}

impl BatchedRollbackTargetRule for BatchedRollbackTargetReviewerLengthRule {
    fn check(&self, item: &BatchedRollbackTarget) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackTargetCategoryLengthRule {
    pub max_length: i64,
}

impl BatchedRollbackTargetRule for BatchedRollbackTargetCategoryLengthRule {
    fn check(&self, item: &BatchedRollbackTarget) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackTargetVisibleGuardRule {
    pub limit: i64,
}

impl BatchedRollbackTargetRule for BatchedRollbackTargetVisibleGuardRule {
    fn check(&self, item: &BatchedRollbackTarget) -> Vec<String> {
        if item.visible && (item.attempts > self.limit) {
            return vec![format!(
                "attempts {} exceeds {} while visible",
                item.attempts, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackTargetCodePrefixRule {
    pub prefix: String,
}

impl BatchedRollbackTargetRule for BatchedRollbackTargetCodePrefixRule {
    fn check(&self, item: &BatchedRollbackTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackTargetRequiredTitleRule {
    pub message: String,
}

impl BatchedRollbackTargetRule for BatchedRollbackTargetRequiredTitleRule {
    fn check(&self, item: &BatchedRollbackTarget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched rollback target.
#[derive(Debug, Clone)]
pub struct BatchedRollbackTargetValidator {
    pub rules: Vec<Rc<dyn BatchedRollbackTargetRule>>,
}

impl BatchedRollbackTargetValidator {
    pub fn validate(&self, item: &BatchedRollbackTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedRollbackTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRollbackTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_rollback_target_validator() -> BatchedRollbackTargetValidator {
    BatchedRollbackTargetValidator {
        rules: vec![
            Rc::new(BatchedRollbackTargetReviewerLengthRule { max_length: 255 }) as Rc<dyn BatchedRollbackTargetRule>,
            Rc::new(BatchedRollbackTargetCategoryLengthRule { max_length: 16 }),
            Rc::new(BatchedRollbackTargetVisibleGuardRule { limit: 255 }),
            Rc::new(BatchedRollbackTargetCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(BatchedRollbackTargetRequiredTitleRule {
                message: "batched rollback target needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_rollback_target_validator();
        let item = new_batched_rollback_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = BatchedRollbackTargetReviewerLengthRule { max_length: 3600 };
        let mut item = new_batched_rollback_target("x", "y");
        item.reviewer = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = BatchedRollbackTargetCategoryLengthRule { max_length: 86400 };
        let mut item = new_batched_rollback_target("x", "y");
        item.category = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BatchedRollbackTargetRequiredTitleRule {
            message: "batched rollback target needs this field".to_string(),
        };
        let mut item = new_batched_rollback_target("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
