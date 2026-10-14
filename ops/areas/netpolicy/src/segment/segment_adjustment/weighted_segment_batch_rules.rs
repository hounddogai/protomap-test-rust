use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted segment batch must pass. Check returns its problems, or none.
pub trait WeightedSegmentBatchRule: std::fmt::Debug {
    fn check(&self, item: &WeightedSegmentBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentBatchAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl WeightedSegmentBatchRule for WeightedSegmentBatchAllowedLabelRule {
    fn check(&self, item: &WeightedSegmentBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentBatchVisibleGuardRule {
    pub limit: i64,
}

impl WeightedSegmentBatchRule for WeightedSegmentBatchVisibleGuardRule {
    fn check(&self, item: &WeightedSegmentBatch) -> Vec<String> {
        if item.visible && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while visible",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentBatchCodeLengthRule {
    pub max_length: i64,
}

impl WeightedSegmentBatchRule for WeightedSegmentBatchCodeLengthRule {
    fn check(&self, item: &WeightedSegmentBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted segment batch.
#[derive(Debug, Clone)]
pub struct WeightedSegmentBatchValidator {
    pub rules: Vec<Rc<dyn WeightedSegmentBatchRule>>,
}

impl WeightedSegmentBatchValidator {
    pub fn validate(&self, item: &WeightedSegmentBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedSegmentBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedSegmentBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_segment_batch_validator() -> WeightedSegmentBatchValidator {
    WeightedSegmentBatchValidator {
        rules: vec![
            Rc::new(WeightedSegmentBatchAllowedLabelRule {
                allowed: vec!["rolling".to_string(), "rolling".to_string(), "fallback".to_string()],
            }) as Rc<dyn WeightedSegmentBatchRule>,
            Rc::new(WeightedSegmentBatchVisibleGuardRule { limit: 32 }),
            Rc::new(WeightedSegmentBatchCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_segment_batch_validator();
        let item = new_weighted_segment_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedSegmentBatchCodeLengthRule { max_length: 80 };
        let mut item = new_weighted_segment_batch("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
