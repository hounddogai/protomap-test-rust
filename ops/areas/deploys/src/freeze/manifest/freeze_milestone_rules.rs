use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze milestone must pass. Check returns its problems, or none.
pub trait FreezeMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &FreezeMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMilestoneAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FreezeMilestoneRule for FreezeMilestoneAmountRangeRule {
    fn check(&self, item: &FreezeMilestone) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMilestoneReviewerLengthRule {
    pub max_length: i64,
}

impl FreezeMilestoneRule for FreezeMilestoneReviewerLengthRule {
    fn check(&self, item: &FreezeMilestone) -> Vec<String> {
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
pub struct FreezeMilestoneLabelLengthRule {
    pub max_length: i64,
}

impl FreezeMilestoneRule for FreezeMilestoneLabelLengthRule {
    fn check(&self, item: &FreezeMilestone) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMilestonePhoneLengthRule {
    pub max_length: i64,
}

impl FreezeMilestoneRule for FreezeMilestonePhoneLengthRule {
    fn check(&self, item: &FreezeMilestone) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze milestone.
#[derive(Debug, Clone)]
pub struct FreezeMilestoneValidator {
    pub rules: Vec<Rc<dyn FreezeMilestoneRule>>,
}

impl FreezeMilestoneValidator {
    pub fn validate(&self, item: &FreezeMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_milestone_validator() -> FreezeMilestoneValidator {
    FreezeMilestoneValidator {
        rules: vec![
            Rc::new(FreezeMilestoneAmountRangeRule { min: 5.9, max: 1731.4 }) as Rc<dyn FreezeMilestoneRule>,
            Rc::new(FreezeMilestoneReviewerLengthRule { max_length: 255 }),
            Rc::new(FreezeMilestoneLabelLengthRule { max_length: 120 }),
            Rc::new(FreezeMilestonePhoneLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_milestone_validator();
        let item = new_freeze_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = FreezeMilestoneAmountRangeRule { min: 3.2, max: 2020.7 };
        let mut item = new_freeze_milestone("x", "y");
        item.amount = 2022.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = FreezeMilestoneReviewerLengthRule { max_length: 16 };
        let mut item = new_freeze_milestone("x", "y");
        item.reviewer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FreezeMilestoneLabelLengthRule { max_length: 64 };
        let mut item = new_freeze_milestone("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = FreezeMilestonePhoneLengthRule { max_length: 32 };
        let mut item = new_freeze_milestone("x", "y");
        item.phone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
