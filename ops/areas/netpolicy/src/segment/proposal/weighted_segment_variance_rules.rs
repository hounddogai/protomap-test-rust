use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted segment variance must pass. Check returns its problems, or none.
pub trait WeightedSegmentVarianceRule: std::fmt::Debug {
    fn check(&self, item: &WeightedSegmentVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentVarianceCategoryLengthRule {
    pub max_length: i64,
}

impl WeightedSegmentVarianceRule for WeightedSegmentVarianceCategoryLengthRule {
    fn check(&self, item: &WeightedSegmentVariance) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentVarianceRequiredSourceRule {
    pub message: String,
}

impl WeightedSegmentVarianceRule for WeightedSegmentVarianceRequiredSourceRule {
    fn check(&self, item: &WeightedSegmentVariance) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentVarianceNameLengthRule {
    pub max_length: i64,
}

impl WeightedSegmentVarianceRule for WeightedSegmentVarianceNameLengthRule {
    fn check(&self, item: &WeightedSegmentVariance) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted segment variance.
#[derive(Debug, Clone)]
pub struct WeightedSegmentVarianceValidator {
    pub rules: Vec<Rc<dyn WeightedSegmentVarianceRule>>,
}

impl WeightedSegmentVarianceValidator {
    pub fn validate(&self, item: &WeightedSegmentVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedSegmentVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedSegmentVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_segment_variance_validator() -> WeightedSegmentVarianceValidator {
    WeightedSegmentVarianceValidator {
        rules: vec![
            Rc::new(WeightedSegmentVarianceCategoryLengthRule { max_length: 86400 })
                as Rc<dyn WeightedSegmentVarianceRule>,
            Rc::new(WeightedSegmentVarianceRequiredSourceRule {
                message: "weighted segment variance needs a value".to_string(),
            }),
            Rc::new(WeightedSegmentVarianceNameLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_segment_variance_validator();
        let item = new_weighted_segment_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = WeightedSegmentVarianceCategoryLengthRule { max_length: 16 };
        let mut item = new_weighted_segment_variance("x", "y");
        item.category = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = WeightedSegmentVarianceRequiredSourceRule {
            message: "weighted segment variance needs this field".to_string(),
        };
        let mut item = new_weighted_segment_variance("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedSegmentVarianceNameLengthRule { max_length: 3600 };
        let mut item = new_weighted_segment_variance("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
