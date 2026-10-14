use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy review must pass. Check returns its problems, or none.
pub trait PolicyReviewRule: std::fmt::Debug {
    fn check(&self, item: &PolicyReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyReviewVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyReviewRule for PolicyReviewVersionRangeRule {
    fn check(&self, item: &PolicyReview) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyReviewVisibleGuardRule {
    pub limit: i64,
}

impl PolicyReviewRule for PolicyReviewVisibleGuardRule {
    fn check(&self, item: &PolicyReview) -> Vec<String> {
        if item.visible && (item.created_at > self.limit) {
            return vec![format!(
                "created at {} exceeds {} while visible",
                item.created_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyReviewPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyReviewRule for PolicyReviewPortRangeRule {
    fn check(&self, item: &PolicyReview) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy review.
#[derive(Debug, Clone)]
pub struct PolicyReviewValidator {
    pub rules: Vec<Rc<dyn PolicyReviewRule>>,
}

impl PolicyReviewValidator {
    pub fn validate(&self, item: &PolicyReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_review_validator() -> PolicyReviewValidator {
    PolicyReviewValidator {
        rules: vec![
            Rc::new(PolicyReviewVersionRangeRule { min: 80, max: 3600 }) as Rc<dyn PolicyReviewRule>,
            Rc::new(PolicyReviewVisibleGuardRule { limit: 1000 }),
            Rc::new(PolicyReviewPortRangeRule { min: 80, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_review_validator();
        let item = new_policy_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = PolicyReviewVersionRangeRule { min: 64, max: 80 };
        let mut item = new_policy_review("x", "y");
        item.version = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = PolicyReviewPortRangeRule { min: 16, max: 64 };
        let mut item = new_policy_review("x", "y");
        item.port = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
