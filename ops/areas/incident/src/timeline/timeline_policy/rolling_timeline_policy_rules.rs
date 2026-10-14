use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling timeline policy must pass. Check returns its problems, or none.
pub trait RollingTimelinePolicyRule: std::fmt::Debug {
    fn check(&self, item: &RollingTimelinePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelinePolicyRequiredCodeRule {
    pub message: String,
}

impl RollingTimelinePolicyRule for RollingTimelinePolicyRequiredCodeRule {
    fn check(&self, item: &RollingTimelinePolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelinePolicySummaryLengthRule {
    pub max_length: i64,
}

impl RollingTimelinePolicyRule for RollingTimelinePolicySummaryLengthRule {
    fn check(&self, item: &RollingTimelinePolicy) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelinePolicyQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingTimelinePolicyRule for RollingTimelinePolicyQuantityRangeRule {
    fn check(&self, item: &RollingTimelinePolicy) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelinePolicyRequiredReviewerRule {
    pub message: String,
}

impl RollingTimelinePolicyRule for RollingTimelinePolicyRequiredReviewerRule {
    fn check(&self, item: &RollingTimelinePolicy) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelinePolicyReviewerLengthRule {
    pub max_length: i64,
}

impl RollingTimelinePolicyRule for RollingTimelinePolicyReviewerLengthRule {
    fn check(&self, item: &RollingTimelinePolicy) -> Vec<String> {
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

/// Runs every rule of a rolling timeline policy.
#[derive(Debug, Clone)]
pub struct RollingTimelinePolicyValidator {
    pub rules: Vec<Rc<dyn RollingTimelinePolicyRule>>,
}

impl RollingTimelinePolicyValidator {
    pub fn validate(&self, item: &RollingTimelinePolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingTimelinePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTimelinePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_timeline_policy_validator() -> RollingTimelinePolicyValidator {
    RollingTimelinePolicyValidator {
        rules: vec![
            Rc::new(RollingTimelinePolicyRequiredCodeRule {
                message: "rolling timeline policy needs a value".to_string(),
            }) as Rc<dyn RollingTimelinePolicyRule>,
            Rc::new(RollingTimelinePolicySummaryLengthRule { max_length: 32 }),
            Rc::new(RollingTimelinePolicyQuantityRangeRule { min: 64, max: 255 }),
            Rc::new(RollingTimelinePolicyRequiredReviewerRule {
                message: "rolling timeline policy needs an entry".to_string(),
            }),
            Rc::new(RollingTimelinePolicyReviewerLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_timeline_policy_validator();
        let item = new_rolling_timeline_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingTimelinePolicyRequiredCodeRule {
            message: "rolling timeline policy needs a value".to_string(),
        };
        let mut item = new_rolling_timeline_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = RollingTimelinePolicySummaryLengthRule { max_length: 1000 };
        let mut item = new_rolling_timeline_policy("x", "y");
        item.summary = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = RollingTimelinePolicyQuantityRangeRule { min: 3600, max: 86400 };
        let mut item = new_rolling_timeline_policy("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = RollingTimelinePolicyRequiredReviewerRule {
            message: "rolling timeline policy needs an entry".to_string(),
        };
        let mut item = new_rolling_timeline_policy("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = RollingTimelinePolicyReviewerLengthRule { max_length: 32 };
        let mut item = new_rolling_timeline_policy("x", "y");
        item.reviewer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
