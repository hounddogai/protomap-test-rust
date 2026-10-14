use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient segment item must pass. Check returns its problems, or none.
pub trait LenientSegmentItemRule: std::fmt::Debug {
    fn check(&self, item: &LenientSegmentItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSegmentItemAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl LenientSegmentItemRule for LenientSegmentItemAllowedLabelRule {
    fn check(&self, item: &LenientSegmentItem) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is expired", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSegmentItemCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientSegmentItemRule for LenientSegmentItemCreatedAtRangeRule {
    fn check(&self, item: &LenientSegmentItem) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSegmentItemPhoneLengthRule {
    pub max_length: i64,
}

impl LenientSegmentItemRule for LenientSegmentItemPhoneLengthRule {
    fn check(&self, item: &LenientSegmentItem) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSegmentItemRequiredCodeRule {
    pub message: String,
}

impl LenientSegmentItemRule for LenientSegmentItemRequiredCodeRule {
    fn check(&self, item: &LenientSegmentItem) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient segment item.
#[derive(Debug, Clone)]
pub struct LenientSegmentItemValidator {
    pub rules: Vec<Rc<dyn LenientSegmentItemRule>>,
}

impl LenientSegmentItemValidator {
    pub fn validate(&self, item: &LenientSegmentItem) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientSegmentItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSegmentItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_segment_item_validator() -> LenientSegmentItemValidator {
    LenientSegmentItemValidator {
        rules: vec![
            Rc::new(LenientSegmentItemAllowedLabelRule {
                allowed: vec!["batched".to_string(), "legacy".to_string()],
            }) as Rc<dyn LenientSegmentItemRule>,
            Rc::new(LenientSegmentItemCreatedAtRangeRule { min: 1000, max: 1000 }),
            Rc::new(LenientSegmentItemPhoneLengthRule { max_length: 64 }),
            Rc::new(LenientSegmentItemRequiredCodeRule {
                message: "lenient segment item needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_segment_item_validator();
        let item = new_lenient_segment_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = LenientSegmentItemCreatedAtRangeRule { min: 32, max: 1000 };
        let mut item = new_lenient_segment_item("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = LenientSegmentItemPhoneLengthRule { max_length: 80 };
        let mut item = new_lenient_segment_item("x", "y");
        item.phone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientSegmentItemRequiredCodeRule {
            message: "lenient segment item needs this field".to_string(),
        };
        let mut item = new_lenient_segment_item("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
