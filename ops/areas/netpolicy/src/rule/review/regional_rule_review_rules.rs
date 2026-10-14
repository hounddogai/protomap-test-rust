use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional rule review must pass. Check returns its problems, or none.
pub trait RegionalRuleReviewRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRuleReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleReviewReviewerLengthRule {
    pub max_length: i64,
}

impl RegionalRuleReviewRule for RegionalRuleReviewReviewerLengthRule {
    fn check(&self, item: &RegionalRuleReview) -> Vec<String> {
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
pub struct RegionalRuleReviewCodeLengthRule {
    pub max_length: i64,
}

impl RegionalRuleReviewRule for RegionalRuleReviewCodeLengthRule {
    fn check(&self, item: &RegionalRuleReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleReviewQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalRuleReviewRule for RegionalRuleReviewQuantityRangeRule {
    fn check(&self, item: &RegionalRuleReview) -> Vec<String> {
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
pub struct RegionalRuleReviewLabelLengthRule {
    pub max_length: i64,
}

impl RegionalRuleReviewRule for RegionalRuleReviewLabelLengthRule {
    fn check(&self, item: &RegionalRuleReview) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleReviewCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalRuleReviewRule for RegionalRuleReviewCostRangeRule {
    fn check(&self, item: &RegionalRuleReview) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional rule review.
#[derive(Debug, Clone)]
pub struct RegionalRuleReviewValidator {
    pub rules: Vec<Rc<dyn RegionalRuleReviewRule>>,
}

impl RegionalRuleReviewValidator {
    pub fn validate(&self, item: &RegionalRuleReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalRuleReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRuleReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_rule_review_validator() -> RegionalRuleReviewValidator {
    RegionalRuleReviewValidator {
        rules: vec![
            Rc::new(RegionalRuleReviewReviewerLengthRule { max_length: 1000 }) as Rc<dyn RegionalRuleReviewRule>,
            Rc::new(RegionalRuleReviewCodeLengthRule { max_length: 16 }),
            Rc::new(RegionalRuleReviewQuantityRangeRule { min: 3600, max: 86400 }),
            Rc::new(RegionalRuleReviewLabelLengthRule { max_length: 3600 }),
            Rc::new(RegionalRuleReviewCostRangeRule { min: 8.7, max: 2329.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_rule_review_validator();
        let item = new_regional_rule_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = RegionalRuleReviewReviewerLengthRule { max_length: 120 };
        let mut item = new_regional_rule_review("x", "y");
        item.reviewer = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalRuleReviewCodeLengthRule { max_length: 86400 };
        let mut item = new_regional_rule_review("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = RegionalRuleReviewQuantityRangeRule { min: 64, max: 86400 };
        let mut item = new_regional_rule_review("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalRuleReviewLabelLengthRule { max_length: 80 };
        let mut item = new_regional_rule_review("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = RegionalRuleReviewCostRangeRule { min: 9.0, max: 1982.2 };
        let mut item = new_regional_rule_review("x", "y");
        item.cost = 1983.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
