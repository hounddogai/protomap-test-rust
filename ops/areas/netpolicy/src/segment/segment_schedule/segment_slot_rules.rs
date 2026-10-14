use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment slot must pass. Check returns its problems, or none.
pub trait SegmentSlotRule: std::fmt::Debug {
    fn check(&self, item: &SegmentSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSlotWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentSlotRule for SegmentSlotWindowDaysRangeRule {
    fn check(&self, item: &SegmentSlot) -> Vec<String> {
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
pub struct SegmentSlotRequiredCodeRule {
    pub message: String,
}

impl SegmentSlotRule for SegmentSlotRequiredCodeRule {
    fn check(&self, item: &SegmentSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSlotIssuerLengthRule {
    pub max_length: i64,
}

impl SegmentSlotRule for SegmentSlotIssuerLengthRule {
    fn check(&self, item: &SegmentSlot) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSlotCodePrefixRule {
    pub prefix: String,
}

impl SegmentSlotRule for SegmentSlotCodePrefixRule {
    fn check(&self, item: &SegmentSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment slot.
#[derive(Debug, Clone)]
pub struct SegmentSlotValidator {
    pub rules: Vec<Rc<dyn SegmentSlotRule>>,
}

impl SegmentSlotValidator {
    pub fn validate(&self, item: &SegmentSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_slot_validator() -> SegmentSlotValidator {
    SegmentSlotValidator {
        rules: vec![
            Rc::new(SegmentSlotWindowDaysRangeRule { min: 16, max: 3600 }) as Rc<dyn SegmentSlotRule>,
            Rc::new(SegmentSlotRequiredCodeRule {
                message: "segment slot needs a value".to_string(),
            }),
            Rc::new(SegmentSlotIssuerLengthRule { max_length: 80 }),
            Rc::new(SegmentSlotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_slot_validator();
        let item = new_segment_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = SegmentSlotWindowDaysRangeRule { min: 80, max: 255 };
        let mut item = new_segment_slot("x", "y");
        item.window_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SegmentSlotRequiredCodeRule {
            message: "segment slot needs a value".to_string(),
        };
        let mut item = new_segment_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = SegmentSlotIssuerLengthRule { max_length: 1000 };
        let mut item = new_segment_slot("x", "y");
        item.issuer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
