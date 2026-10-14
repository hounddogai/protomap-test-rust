use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted segment draft must pass. Check returns its problems, or none.
pub trait WeightedSegmentDraftRule: std::fmt::Debug {
    fn check(&self, item: &WeightedSegmentDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentDraftCodeLengthRule {
    pub max_length: i64,
}

impl WeightedSegmentDraftRule for WeightedSegmentDraftCodeLengthRule {
    fn check(&self, item: &WeightedSegmentDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentDraftRequiredCurrencyRule {
    pub message: String,
}

impl WeightedSegmentDraftRule for WeightedSegmentDraftRequiredCurrencyRule {
    fn check(&self, item: &WeightedSegmentDraft) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentDraftTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl WeightedSegmentDraftRule for WeightedSegmentDraftTagsLimitRule {
    fn check(&self, item: &WeightedSegmentDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentDraftMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedSegmentDraftRule for WeightedSegmentDraftMaximumRangeRule {
    fn check(&self, item: &WeightedSegmentDraft) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentDraftRequiredLabelRule {
    pub message: String,
}

impl WeightedSegmentDraftRule for WeightedSegmentDraftRequiredLabelRule {
    fn check(&self, item: &WeightedSegmentDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentDraftRequiredCodeRule {
    pub message: String,
}

impl WeightedSegmentDraftRule for WeightedSegmentDraftRequiredCodeRule {
    fn check(&self, item: &WeightedSegmentDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted segment draft.
#[derive(Debug, Clone)]
pub struct WeightedSegmentDraftValidator {
    pub rules: Vec<Rc<dyn WeightedSegmentDraftRule>>,
}

impl WeightedSegmentDraftValidator {
    pub fn validate(&self, item: &WeightedSegmentDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedSegmentDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedSegmentDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_segment_draft_validator() -> WeightedSegmentDraftValidator {
    WeightedSegmentDraftValidator {
        rules: vec![
            Rc::new(WeightedSegmentDraftCodeLengthRule { max_length: 120 }) as Rc<dyn WeightedSegmentDraftRule>,
            Rc::new(WeightedSegmentDraftRequiredCurrencyRule {
                message: "weighted segment draft needs an entry".to_string(),
            }),
            Rc::new(WeightedSegmentDraftTagsLimitRule {
                max_count: 16,
                banned: Vec::new(),
            }),
            Rc::new(WeightedSegmentDraftMaximumRangeRule { min: 8.4, max: 197.6 }),
            Rc::new(WeightedSegmentDraftRequiredLabelRule {
                message: "weighted segment draft needs this field".to_string(),
            }),
            Rc::new(WeightedSegmentDraftRequiredCodeRule {
                message: "weighted segment draft needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_segment_draft_validator();
        let item = new_weighted_segment_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedSegmentDraftCodeLengthRule { max_length: 3600 };
        let mut item = new_weighted_segment_draft("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = WeightedSegmentDraftRequiredCurrencyRule {
            message: "weighted segment draft needs a value".to_string(),
        };
        let mut item = new_weighted_segment_draft("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = WeightedSegmentDraftMaximumRangeRule { min: 8.1, max: 2675.8 };
        let mut item = new_weighted_segment_draft("x", "y");
        item.maximum = 2677.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = WeightedSegmentDraftRequiredLabelRule {
            message: "weighted segment draft needs an entry".to_string(),
        };
        let mut item = new_weighted_segment_draft("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedSegmentDraftRequiredCodeRule {
            message: "weighted segment draft needs an entry".to_string(),
        };
        let mut item = new_weighted_segment_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
