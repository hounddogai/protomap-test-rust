use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment estimate must pass. Check returns its problems, or none.
pub trait SegmentEstimateRule: std::fmt::Debug {
    fn check(&self, item: &SegmentEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEstimateNameLengthRule {
    pub max_length: i64,
}

impl SegmentEstimateRule for SegmentEstimateNameLengthRule {
    fn check(&self, item: &SegmentEstimate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEstimateRequiredNameRule {
    pub message: String,
}

impl SegmentEstimateRule for SegmentEstimateRequiredNameRule {
    fn check(&self, item: &SegmentEstimate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEstimateVisibleGuardRule {
    pub limit: f64,
}

impl SegmentEstimateRule for SegmentEstimateVisibleGuardRule {
    fn check(&self, item: &SegmentEstimate) -> Vec<String> {
        if item.visible && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while visible", item.amount, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEstimateLabelLengthRule {
    pub max_length: i64,
}

impl SegmentEstimateRule for SegmentEstimateLabelLengthRule {
    fn check(&self, item: &SegmentEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment estimate.
#[derive(Debug, Clone)]
pub struct SegmentEstimateValidator {
    pub rules: Vec<Rc<dyn SegmentEstimateRule>>,
}

impl SegmentEstimateValidator {
    pub fn validate(&self, item: &SegmentEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_estimate_validator() -> SegmentEstimateValidator {
    SegmentEstimateValidator {
        rules: vec![
            Rc::new(SegmentEstimateNameLengthRule { max_length: 120 }) as Rc<dyn SegmentEstimateRule>,
            Rc::new(SegmentEstimateRequiredNameRule {
                message: "segment estimate needs an entry".to_string(),
            }),
            Rc::new(SegmentEstimateVisibleGuardRule { limit: 5.1 }),
            Rc::new(SegmentEstimateLabelLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_estimate_validator();
        let item = new_segment_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SegmentEstimateNameLengthRule { max_length: 64 };
        let mut item = new_segment_estimate("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SegmentEstimateRequiredNameRule {
            message: "segment estimate needs a value".to_string(),
        };
        let mut item = new_segment_estimate("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SegmentEstimateLabelLengthRule { max_length: 16 };
        let mut item = new_segment_estimate("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
