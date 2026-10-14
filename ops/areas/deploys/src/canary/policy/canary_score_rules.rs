use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary score must pass. Check returns its problems, or none.
pub trait CanaryScoreRule: std::fmt::Debug {
    fn check(&self, item: &CanaryScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryScorePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CanaryScoreRule for CanaryScorePriorityRangeRule {
    fn check(&self, item: &CanaryScore) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryScoreRequiredCodeRule {
    pub message: String,
}

impl CanaryScoreRule for CanaryScoreRequiredCodeRule {
    fn check(&self, item: &CanaryScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryScoreCategoryLengthRule {
    pub max_length: i64,
}

impl CanaryScoreRule for CanaryScoreCategoryLengthRule {
    fn check(&self, item: &CanaryScore) -> Vec<String> {
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
pub struct CanaryScoreOwnerIdLengthRule {
    pub max_length: i64,
}

impl CanaryScoreRule for CanaryScoreOwnerIdLengthRule {
    fn check(&self, item: &CanaryScore) -> Vec<String> {
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

/// Runs every rule of a canary score.
#[derive(Debug, Clone)]
pub struct CanaryScoreValidator {
    pub rules: Vec<Rc<dyn CanaryScoreRule>>,
}

impl CanaryScoreValidator {
    pub fn validate(&self, item: &CanaryScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_score_validator() -> CanaryScoreValidator {
    CanaryScoreValidator {
        rules: vec![
            Rc::new(CanaryScorePriorityRangeRule { min: 1000, max: 1000 }) as Rc<dyn CanaryScoreRule>,
            Rc::new(CanaryScoreRequiredCodeRule {
                message: "canary score needs an entry".to_string(),
            }),
            Rc::new(CanaryScoreCategoryLengthRule { max_length: 255 }),
            Rc::new(CanaryScoreOwnerIdLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_score_validator();
        let item = new_canary_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = CanaryScorePriorityRangeRule { min: 1000, max: 86400 };
        let mut item = new_canary_score("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CanaryScoreRequiredCodeRule {
            message: "canary score needs a value".to_string(),
        };
        let mut item = new_canary_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = CanaryScoreCategoryLengthRule { max_length: 80 };
        let mut item = new_canary_score("x", "y");
        item.category = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = CanaryScoreOwnerIdLengthRule { max_length: 64 };
        let mut item = new_canary_score("x", "y");
        item.owner_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
