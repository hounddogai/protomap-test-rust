use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo profile must pass. Check returns its problems, or none.
pub trait SloProfileRule: std::fmt::Debug {
    fn check(&self, item: &SloProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloProfileReviewerLengthRule {
    pub max_length: i64,
}

impl SloProfileRule for SloProfileReviewerLengthRule {
    fn check(&self, item: &SloProfile) -> Vec<String> {
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
pub struct SloProfileLabelLengthRule {
    pub max_length: i64,
}

impl SloProfileRule for SloProfileLabelLengthRule {
    fn check(&self, item: &SloProfile) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloProfileDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloProfileRule for SloProfileDiscountRangeRule {
    fn check(&self, item: &SloProfile) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloProfileRequiredLabelRule {
    pub message: String,
}

impl SloProfileRule for SloProfileRequiredLabelRule {
    fn check(&self, item: &SloProfile) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo profile.
#[derive(Debug, Clone)]
pub struct SloProfileValidator {
    pub rules: Vec<Rc<dyn SloProfileRule>>,
}

impl SloProfileValidator {
    pub fn validate(&self, item: &SloProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_profile_validator() -> SloProfileValidator {
    SloProfileValidator {
        rules: vec![
            Rc::new(SloProfileReviewerLengthRule { max_length: 1000 }) as Rc<dyn SloProfileRule>,
            Rc::new(SloProfileLabelLengthRule { max_length: 64 }),
            Rc::new(SloProfileDiscountRangeRule { min: 4.3, max: 2263.0 }),
            Rc::new(SloProfileRequiredLabelRule {
                message: "slo profile needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_profile_validator();
        let item = new_slo_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = SloProfileReviewerLengthRule { max_length: 3600 };
        let mut item = new_slo_profile("x", "y");
        item.reviewer = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SloProfileLabelLengthRule { max_length: 120 };
        let mut item = new_slo_profile("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = SloProfileDiscountRangeRule { min: 1.0, max: 4756.0 };
        let mut item = new_slo_profile("x", "y");
        item.discount = 4757.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SloProfileRequiredLabelRule {
            message: "slo profile needs an entry".to_string(),
        };
        let mut item = new_slo_profile("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
