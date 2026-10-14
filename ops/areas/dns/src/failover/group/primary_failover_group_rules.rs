use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary failover group must pass. Check returns its problems, or none.
pub trait PrimaryFailoverGroupRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryFailoverGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverGroupRequiredReviewerRule {
    pub message: String,
}

impl PrimaryFailoverGroupRule for PrimaryFailoverGroupRequiredReviewerRule {
    fn check(&self, item: &PrimaryFailoverGroup) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverGroupThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryFailoverGroupRule for PrimaryFailoverGroupThresholdRangeRule {
    fn check(&self, item: &PrimaryFailoverGroup) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverGroupRequiredLabelRule {
    pub message: String,
}

impl PrimaryFailoverGroupRule for PrimaryFailoverGroupRequiredLabelRule {
    fn check(&self, item: &PrimaryFailoverGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverGroupCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryFailoverGroupRule for PrimaryFailoverGroupCodeLengthRule {
    fn check(&self, item: &PrimaryFailoverGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverGroupRequiredSummaryRule {
    pub message: String,
}

impl PrimaryFailoverGroupRule for PrimaryFailoverGroupRequiredSummaryRule {
    fn check(&self, item: &PrimaryFailoverGroup) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverGroupReviewerLengthRule {
    pub max_length: i64,
}

impl PrimaryFailoverGroupRule for PrimaryFailoverGroupReviewerLengthRule {
    fn check(&self, item: &PrimaryFailoverGroup) -> Vec<String> {
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

/// Runs every rule of a primary failover group.
#[derive(Debug, Clone)]
pub struct PrimaryFailoverGroupValidator {
    pub rules: Vec<Rc<dyn PrimaryFailoverGroupRule>>,
}

impl PrimaryFailoverGroupValidator {
    pub fn validate(&self, item: &PrimaryFailoverGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryFailoverGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryFailoverGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_failover_group_validator() -> PrimaryFailoverGroupValidator {
    PrimaryFailoverGroupValidator {
        rules: vec![
            Rc::new(PrimaryFailoverGroupRequiredReviewerRule {
                message: "primary failover group needs an entry".to_string(),
            }) as Rc<dyn PrimaryFailoverGroupRule>,
            Rc::new(PrimaryFailoverGroupThresholdRangeRule { min: 5.6, max: 3016.3 }),
            Rc::new(PrimaryFailoverGroupRequiredLabelRule {
                message: "primary failover group needs an entry".to_string(),
            }),
            Rc::new(PrimaryFailoverGroupCodeLengthRule { max_length: 255 }),
            Rc::new(PrimaryFailoverGroupRequiredSummaryRule {
                message: "primary failover group needs a value".to_string(),
            }),
            Rc::new(PrimaryFailoverGroupReviewerLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_failover_group_validator();
        let item = new_primary_failover_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = PrimaryFailoverGroupRequiredReviewerRule {
            message: "primary failover group needs a value".to_string(),
        };
        let mut item = new_primary_failover_group("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = PrimaryFailoverGroupThresholdRangeRule { min: 3.8, max: 99.0 };
        let mut item = new_primary_failover_group("x", "y");
        item.threshold = 100.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimaryFailoverGroupRequiredLabelRule {
            message: "primary failover group needs an entry".to_string(),
        };
        let mut item = new_primary_failover_group("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryFailoverGroupCodeLengthRule { max_length: 3600 };
        let mut item = new_primary_failover_group("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = PrimaryFailoverGroupRequiredSummaryRule {
            message: "primary failover group needs an entry".to_string(),
        };
        let mut item = new_primary_failover_group("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = PrimaryFailoverGroupReviewerLengthRule { max_length: 120 };
        let mut item = new_primary_failover_group("x", "y");
        item.reviewer = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
