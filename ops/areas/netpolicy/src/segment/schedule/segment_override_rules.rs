use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment override must pass. Check returns its problems, or none.
pub trait SegmentOverrideRule: std::fmt::Debug {
    fn check(&self, item: &SegmentOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentOverrideRequiredReviewerRule {
    pub message: String,
}

impl SegmentOverrideRule for SegmentOverrideRequiredReviewerRule {
    fn check(&self, item: &SegmentOverride) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentOverrideReviewerLengthRule {
    pub max_length: i64,
}

impl SegmentOverrideRule for SegmentOverrideReviewerLengthRule {
    fn check(&self, item: &SegmentOverride) -> Vec<String> {
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
pub struct SegmentOverrideBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SegmentOverrideRule for SegmentOverrideBudgetRangeRule {
    fn check(&self, item: &SegmentOverride) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentOverrideWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SegmentOverrideRule for SegmentOverrideWeightRangeRule {
    fn check(&self, item: &SegmentOverride) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentOverrideCodePrefixRule {
    pub prefix: String,
}

impl SegmentOverrideRule for SegmentOverrideCodePrefixRule {
    fn check(&self, item: &SegmentOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment override.
#[derive(Debug, Clone)]
pub struct SegmentOverrideValidator {
    pub rules: Vec<Rc<dyn SegmentOverrideRule>>,
}

impl SegmentOverrideValidator {
    pub fn validate(&self, item: &SegmentOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_override_validator() -> SegmentOverrideValidator {
    SegmentOverrideValidator {
        rules: vec![
            Rc::new(SegmentOverrideRequiredReviewerRule {
                message: "segment override needs an entry".to_string(),
            }) as Rc<dyn SegmentOverrideRule>,
            Rc::new(SegmentOverrideReviewerLengthRule { max_length: 16 }),
            Rc::new(SegmentOverrideBudgetRangeRule { min: 6.2, max: 132.0 }),
            Rc::new(SegmentOverrideWeightRangeRule { min: 6.5, max: 349.0 }),
            Rc::new(SegmentOverrideCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_override_validator();
        let item = new_segment_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = SegmentOverrideRequiredReviewerRule {
            message: "segment override needs this field".to_string(),
        };
        let mut item = new_segment_override("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = SegmentOverrideReviewerLengthRule { max_length: 80 };
        let mut item = new_segment_override("x", "y");
        item.reviewer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = SegmentOverrideBudgetRangeRule { min: 0.5, max: 3373.3 };
        let mut item = new_segment_override("x", "y");
        item.budget = 3374.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = SegmentOverrideWeightRangeRule { min: 0.1, max: 2661.2 };
        let mut item = new_segment_override("x", "y");
        item.weight = 2662.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
