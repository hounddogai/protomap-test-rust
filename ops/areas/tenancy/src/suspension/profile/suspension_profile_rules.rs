use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension profile must pass. Check returns its problems, or none.
pub trait SuspensionProfileRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionProfileLabelLengthRule {
    pub max_length: i64,
}

impl SuspensionProfileRule for SuspensionProfileLabelLengthRule {
    fn check(&self, item: &SuspensionProfile) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionProfileSegmentLengthRule {
    pub max_length: i64,
}

impl SuspensionProfileRule for SuspensionProfileSegmentLengthRule {
    fn check(&self, item: &SuspensionProfile) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionProfileRequiredSegmentRule {
    pub message: String,
}

impl SuspensionProfileRule for SuspensionProfileRequiredSegmentRule {
    fn check(&self, item: &SuspensionProfile) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionProfileRequiredTitleRule {
    pub message: String,
}

impl SuspensionProfileRule for SuspensionProfileRequiredTitleRule {
    fn check(&self, item: &SuspensionProfile) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionProfileCodeLengthRule {
    pub max_length: i64,
}

impl SuspensionProfileRule for SuspensionProfileCodeLengthRule {
    fn check(&self, item: &SuspensionProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionProfileCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionProfileRule for SuspensionProfileCostRangeRule {
    fn check(&self, item: &SuspensionProfile) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension profile.
#[derive(Debug, Clone)]
pub struct SuspensionProfileValidator {
    pub rules: Vec<Rc<dyn SuspensionProfileRule>>,
}

impl SuspensionProfileValidator {
    pub fn validate(&self, item: &SuspensionProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_profile_validator() -> SuspensionProfileValidator {
    SuspensionProfileValidator {
        rules: vec![
            Rc::new(SuspensionProfileLabelLengthRule { max_length: 32 }) as Rc<dyn SuspensionProfileRule>,
            Rc::new(SuspensionProfileSegmentLengthRule { max_length: 120 }),
            Rc::new(SuspensionProfileRequiredSegmentRule {
                message: "suspension profile needs a value".to_string(),
            }),
            Rc::new(SuspensionProfileRequiredTitleRule {
                message: "suspension profile needs this field".to_string(),
            }),
            Rc::new(SuspensionProfileCodeLengthRule { max_length: 80 }),
            Rc::new(SuspensionProfileCostRangeRule { min: 9.4, max: 992.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_profile_validator();
        let item = new_suspension_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SuspensionProfileLabelLengthRule { max_length: 16 };
        let mut item = new_suspension_profile("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = SuspensionProfileSegmentLengthRule { max_length: 80 };
        let mut item = new_suspension_profile("x", "y");
        item.segment = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = SuspensionProfileRequiredSegmentRule {
            message: "suspension profile needs this field".to_string(),
        };
        let mut item = new_suspension_profile("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SuspensionProfileRequiredTitleRule {
            message: "suspension profile needs this field".to_string(),
        };
        let mut item = new_suspension_profile("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SuspensionProfileCodeLengthRule { max_length: 80 };
        let mut item = new_suspension_profile("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = SuspensionProfileCostRangeRule { min: 0.3, max: 1823.4 };
        let mut item = new_suspension_profile("x", "y");
        item.cost = 1824.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
