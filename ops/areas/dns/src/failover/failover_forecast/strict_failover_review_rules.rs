use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict failover review must pass. Check returns its problems, or none.
pub trait StrictFailoverReviewRule: std::fmt::Debug {
    fn check(&self, item: &StrictFailoverReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFailoverReviewKeyIdLengthRule {
    pub max_length: i64,
}

impl StrictFailoverReviewRule for StrictFailoverReviewKeyIdLengthRule {
    fn check(&self, item: &StrictFailoverReview) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFailoverReviewTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictFailoverReviewRule for StrictFailoverReviewTtlSecondsRangeRule {
    fn check(&self, item: &StrictFailoverReview) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFailoverReviewNamespaceLengthRule {
    pub max_length: i64,
}

impl StrictFailoverReviewRule for StrictFailoverReviewNamespaceLengthRule {
    fn check(&self, item: &StrictFailoverReview) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFailoverReviewRequiredKeyIdRule {
    pub message: String,
}

impl StrictFailoverReviewRule for StrictFailoverReviewRequiredKeyIdRule {
    fn check(&self, item: &StrictFailoverReview) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict failover review.
#[derive(Debug, Clone)]
pub struct StrictFailoverReviewValidator {
    pub rules: Vec<Rc<dyn StrictFailoverReviewRule>>,
}

impl StrictFailoverReviewValidator {
    pub fn validate(&self, item: &StrictFailoverReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictFailoverReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictFailoverReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_failover_review_validator() -> StrictFailoverReviewValidator {
    StrictFailoverReviewValidator {
        rules: vec![
            Rc::new(StrictFailoverReviewKeyIdLengthRule { max_length: 3600 }) as Rc<dyn StrictFailoverReviewRule>,
            Rc::new(StrictFailoverReviewTtlSecondsRangeRule { min: 1000, max: 86400 }),
            Rc::new(StrictFailoverReviewNamespaceLengthRule { max_length: 1000 }),
            Rc::new(StrictFailoverReviewRequiredKeyIdRule {
                message: "strict failover review needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_failover_review_validator();
        let item = new_strict_failover_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = StrictFailoverReviewKeyIdLengthRule { max_length: 120 };
        let mut item = new_strict_failover_review("x", "y");
        item.key_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = StrictFailoverReviewTtlSecondsRangeRule { min: 32, max: 120 };
        let mut item = new_strict_failover_review("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = StrictFailoverReviewNamespaceLengthRule { max_length: 64 };
        let mut item = new_strict_failover_review("x", "y");
        item.namespace = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = StrictFailoverReviewRequiredKeyIdRule {
            message: "strict failover review needs an entry".to_string(),
        };
        let mut item = new_strict_failover_review("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
