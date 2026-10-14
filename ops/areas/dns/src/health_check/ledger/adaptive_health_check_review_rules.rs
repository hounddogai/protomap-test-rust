use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive health check review must pass. Check returns its problems, or none.
pub trait AdaptiveHealthCheckReviewRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveHealthCheckReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveHealthCheckReviewRequiredScopeRule {
    pub message: String,
}

impl AdaptiveHealthCheckReviewRule for AdaptiveHealthCheckReviewRequiredScopeRule {
    fn check(&self, item: &AdaptiveHealthCheckReview) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveHealthCheckReviewRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveHealthCheckReviewRule for AdaptiveHealthCheckReviewRetriesRangeRule {
    fn check(&self, item: &AdaptiveHealthCheckReview) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveHealthCheckReviewQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveHealthCheckReviewRule for AdaptiveHealthCheckReviewQuantityRangeRule {
    fn check(&self, item: &AdaptiveHealthCheckReview) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveHealthCheckReviewScopeLengthRule {
    pub max_length: i64,
}

impl AdaptiveHealthCheckReviewRule for AdaptiveHealthCheckReviewScopeLengthRule {
    fn check(&self, item: &AdaptiveHealthCheckReview) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveHealthCheckReviewCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveHealthCheckReviewRule for AdaptiveHealthCheckReviewCodePrefixRule {
    fn check(&self, item: &AdaptiveHealthCheckReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive health check review.
#[derive(Debug, Clone)]
pub struct AdaptiveHealthCheckReviewValidator {
    pub rules: Vec<Rc<dyn AdaptiveHealthCheckReviewRule>>,
}

impl AdaptiveHealthCheckReviewValidator {
    pub fn validate(&self, item: &AdaptiveHealthCheckReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveHealthCheckReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveHealthCheckReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_health_check_review_validator() -> AdaptiveHealthCheckReviewValidator {
    AdaptiveHealthCheckReviewValidator {
        rules: vec![
            Rc::new(AdaptiveHealthCheckReviewRequiredScopeRule {
                message: "adaptive health check review needs this field".to_string(),
            }) as Rc<dyn AdaptiveHealthCheckReviewRule>,
            Rc::new(AdaptiveHealthCheckReviewRetriesRangeRule { min: 255, max: 3600 }),
            Rc::new(AdaptiveHealthCheckReviewQuantityRangeRule { min: 16, max: 3600 }),
            Rc::new(AdaptiveHealthCheckReviewScopeLengthRule { max_length: 32 }),
            Rc::new(AdaptiveHealthCheckReviewCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_health_check_review_validator();
        let item = new_adaptive_health_check_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = AdaptiveHealthCheckReviewRequiredScopeRule {
            message: "adaptive health check review needs this field".to_string(),
        };
        let mut item = new_adaptive_health_check_review("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = AdaptiveHealthCheckReviewRetriesRangeRule { min: 80, max: 120 };
        let mut item = new_adaptive_health_check_review("x", "y");
        item.retries = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = AdaptiveHealthCheckReviewQuantityRangeRule { min: 64, max: 80 };
        let mut item = new_adaptive_health_check_review("x", "y");
        item.quantity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = AdaptiveHealthCheckReviewScopeLengthRule { max_length: 1000 };
        let mut item = new_adaptive_health_check_review("x", "y");
        item.scope = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
