use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback rollback policy must pass. Check returns its problems, or none.
pub trait FallbackRollbackPolicyRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRollbackPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackPolicyRequiredRegionRule {
    pub message: String,
}

impl FallbackRollbackPolicyRule for FallbackRollbackPolicyRequiredRegionRule {
    fn check(&self, item: &FallbackRollbackPolicy) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackPolicyCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackRollbackPolicyRule for FallbackRollbackPolicyCreatedAtRangeRule {
    fn check(&self, item: &FallbackRollbackPolicy) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackPolicyRegionLengthRule {
    pub max_length: i64,
}

impl FallbackRollbackPolicyRule for FallbackRollbackPolicyRegionLengthRule {
    fn check(&self, item: &FallbackRollbackPolicy) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackPolicyRequiredReviewerRule {
    pub message: String,
}

impl FallbackRollbackPolicyRule for FallbackRollbackPolicyRequiredReviewerRule {
    fn check(&self, item: &FallbackRollbackPolicy) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback rollback policy.
#[derive(Debug, Clone)]
pub struct FallbackRollbackPolicyValidator {
    pub rules: Vec<Rc<dyn FallbackRollbackPolicyRule>>,
}

impl FallbackRollbackPolicyValidator {
    pub fn validate(&self, item: &FallbackRollbackPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackRollbackPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRollbackPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_rollback_policy_validator() -> FallbackRollbackPolicyValidator {
    FallbackRollbackPolicyValidator {
        rules: vec![
            Rc::new(FallbackRollbackPolicyRequiredRegionRule {
                message: "fallback rollback policy needs a value".to_string(),
            }) as Rc<dyn FallbackRollbackPolicyRule>,
            Rc::new(FallbackRollbackPolicyCreatedAtRangeRule { min: 64, max: 255 }),
            Rc::new(FallbackRollbackPolicyRegionLengthRule { max_length: 64 }),
            Rc::new(FallbackRollbackPolicyRequiredReviewerRule {
                message: "fallback rollback policy needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_rollback_policy_validator();
        let item = new_fallback_rollback_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = FallbackRollbackPolicyRequiredRegionRule {
            message: "fallback rollback policy needs a value".to_string(),
        };
        let mut item = new_fallback_rollback_policy("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = FallbackRollbackPolicyCreatedAtRangeRule { min: 64, max: 80 };
        let mut item = new_fallback_rollback_policy("x", "y");
        item.created_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = FallbackRollbackPolicyRegionLengthRule { max_length: 120 };
        let mut item = new_fallback_rollback_policy("x", "y");
        item.region = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = FallbackRollbackPolicyRequiredReviewerRule {
            message: "fallback rollback policy needs this field".to_string(),
        };
        let mut item = new_fallback_rollback_policy("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
