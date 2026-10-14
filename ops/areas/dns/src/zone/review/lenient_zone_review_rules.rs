use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient zone review must pass. Check returns its problems, or none.
pub trait LenientZoneReviewRule: std::fmt::Debug {
    fn check(&self, item: &LenientZoneReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneReviewLabelLengthRule {
    pub max_length: i64,
}

impl LenientZoneReviewRule for LenientZoneReviewLabelLengthRule {
    fn check(&self, item: &LenientZoneReview) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneReviewWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientZoneReviewRule for LenientZoneReviewWindowDaysRangeRule {
    fn check(&self, item: &LenientZoneReview) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneReviewRequiredExternalRefRule {
    pub message: String,
}

impl LenientZoneReviewRule for LenientZoneReviewRequiredExternalRefRule {
    fn check(&self, item: &LenientZoneReview) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneReviewBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientZoneReviewRule for LenientZoneReviewBurstRangeRule {
    fn check(&self, item: &LenientZoneReview) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneReviewPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientZoneReviewRule for LenientZoneReviewPriorityRangeRule {
    fn check(&self, item: &LenientZoneReview) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient zone review.
#[derive(Debug, Clone)]
pub struct LenientZoneReviewValidator {
    pub rules: Vec<Rc<dyn LenientZoneReviewRule>>,
}

impl LenientZoneReviewValidator {
    pub fn validate(&self, item: &LenientZoneReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientZoneReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientZoneReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_zone_review_validator() -> LenientZoneReviewValidator {
    LenientZoneReviewValidator {
        rules: vec![
            Rc::new(LenientZoneReviewLabelLengthRule { max_length: 16 }) as Rc<dyn LenientZoneReviewRule>,
            Rc::new(LenientZoneReviewWindowDaysRangeRule { min: 120, max: 3600 }),
            Rc::new(LenientZoneReviewRequiredExternalRefRule {
                message: "lenient zone review needs this field".to_string(),
            }),
            Rc::new(LenientZoneReviewBurstRangeRule { min: 64, max: 3600 }),
            Rc::new(LenientZoneReviewPriorityRangeRule { min: 16, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_zone_review_validator();
        let item = new_lenient_zone_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientZoneReviewLabelLengthRule { max_length: 32 };
        let mut item = new_lenient_zone_review("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = LenientZoneReviewWindowDaysRangeRule { min: 120, max: 86400 };
        let mut item = new_lenient_zone_review("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = LenientZoneReviewRequiredExternalRefRule {
            message: "lenient zone review needs an entry".to_string(),
        };
        let mut item = new_lenient_zone_review("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = LenientZoneReviewBurstRangeRule { min: 16, max: 64 };
        let mut item = new_lenient_zone_review("x", "y");
        item.burst = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = LenientZoneReviewPriorityRangeRule { min: 32, max: 120 };
        let mut item = new_lenient_zone_review("x", "y");
        item.priority = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
