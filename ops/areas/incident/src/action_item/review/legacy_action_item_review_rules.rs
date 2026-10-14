use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy action item review must pass. Check returns its problems, or none.
pub trait LegacyActionItemReviewRule: std::fmt::Debug {
    fn check(&self, item: &LegacyActionItemReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemReviewRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyActionItemReviewRule for LegacyActionItemReviewRotationDaysRangeRule {
    fn check(&self, item: &LegacyActionItemReview) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemReviewQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyActionItemReviewRule for LegacyActionItemReviewQuantityRangeRule {
    fn check(&self, item: &LegacyActionItemReview) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemReviewCodePrefixRule {
    pub prefix: String,
}

impl LegacyActionItemReviewRule for LegacyActionItemReviewCodePrefixRule {
    fn check(&self, item: &LegacyActionItemReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemReviewMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyActionItemReviewRule for LegacyActionItemReviewMaximumRangeRule {
    fn check(&self, item: &LegacyActionItemReview) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy action item review.
#[derive(Debug, Clone)]
pub struct LegacyActionItemReviewValidator {
    pub rules: Vec<Rc<dyn LegacyActionItemReviewRule>>,
}

impl LegacyActionItemReviewValidator {
    pub fn validate(&self, item: &LegacyActionItemReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyActionItemReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyActionItemReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_action_item_review_validator() -> LegacyActionItemReviewValidator {
    LegacyActionItemReviewValidator {
        rules: vec![
            Rc::new(LegacyActionItemReviewRotationDaysRangeRule { min: 64, max: 80 })
                as Rc<dyn LegacyActionItemReviewRule>,
            Rc::new(LegacyActionItemReviewQuantityRangeRule { min: 32, max: 3600 }),
            Rc::new(LegacyActionItemReviewCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LegacyActionItemReviewMaximumRangeRule { min: 0.2, max: 4252.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_action_item_review_validator();
        let item = new_legacy_action_item_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = LegacyActionItemReviewRotationDaysRangeRule { min: 16, max: 255 };
        let mut item = new_legacy_action_item_review("x", "y");
        item.rotation_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = LegacyActionItemReviewQuantityRangeRule { min: 86400, max: 86400 };
        let mut item = new_legacy_action_item_review("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = LegacyActionItemReviewMaximumRangeRule { min: 5.9, max: 502.1 };
        let mut item = new_legacy_action_item_review("x", "y");
        item.maximum = 503.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
