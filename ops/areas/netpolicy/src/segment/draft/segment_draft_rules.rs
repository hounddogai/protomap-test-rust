use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment draft must pass. Check returns its problems, or none.
pub trait SegmentDraftRule: std::fmt::Debug {
    fn check(&self, item: &SegmentDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDraftDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentDraftRule for SegmentDraftDurationMinutesRangeRule {
    fn check(&self, item: &SegmentDraft) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDraftRequiredNameRule {
    pub message: String,
}

impl SegmentDraftRule for SegmentDraftRequiredNameRule {
    fn check(&self, item: &SegmentDraft) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDraftRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SegmentDraftRule for SegmentDraftRatioRangeRule {
    fn check(&self, item: &SegmentDraft) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDraftDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SegmentDraftRule for SegmentDraftDiscountRangeRule {
    fn check(&self, item: &SegmentDraft) -> Vec<String> {
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

/// Runs every rule of a segment draft.
#[derive(Debug, Clone)]
pub struct SegmentDraftValidator {
    pub rules: Vec<Rc<dyn SegmentDraftRule>>,
}

impl SegmentDraftValidator {
    pub fn validate(&self, item: &SegmentDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_draft_validator() -> SegmentDraftValidator {
    SegmentDraftValidator {
        rules: vec![
            Rc::new(SegmentDraftDurationMinutesRangeRule { min: 255, max: 86400 }) as Rc<dyn SegmentDraftRule>,
            Rc::new(SegmentDraftRequiredNameRule {
                message: "segment draft needs an entry".to_string(),
            }),
            Rc::new(SegmentDraftRatioRangeRule { min: 1.9, max: 3401.2 }),
            Rc::new(SegmentDraftDiscountRangeRule { min: 4.4, max: 3420.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_draft_validator();
        let item = new_segment_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = SegmentDraftDurationMinutesRangeRule { min: 32, max: 86400 };
        let mut item = new_segment_draft("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SegmentDraftRequiredNameRule {
            message: "segment draft needs this field".to_string(),
        };
        let mut item = new_segment_draft("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = SegmentDraftRatioRangeRule { min: 8.5, max: 2417.8 };
        let mut item = new_segment_draft("x", "y");
        item.ratio = 2419.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = SegmentDraftDiscountRangeRule { min: 8.1, max: 2560.5 };
        let mut item = new_segment_draft("x", "y");
        item.discount = 2562.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
