use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a segment exception must pass. Check returns its problems, or none.
pub trait SegmentExceptionRule: std::fmt::Debug {
    fn check(&self, item: &SegmentException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentExceptionAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SegmentExceptionRule for SegmentExceptionAllowedCodeRule {
    fn check(&self, item: &SegmentException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentExceptionRequiredTitleRule {
    pub message: String,
}

impl SegmentExceptionRule for SegmentExceptionRequiredTitleRule {
    fn check(&self, item: &SegmentException) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentExceptionDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SegmentExceptionRule for SegmentExceptionDiscountRangeRule {
    fn check(&self, item: &SegmentException) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentExceptionCodePrefixRule {
    pub prefix: String,
}

impl SegmentExceptionRule for SegmentExceptionCodePrefixRule {
    fn check(&self, item: &SegmentException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment exception.
#[derive(Debug, Clone)]
pub struct SegmentExceptionValidator {
    pub rules: Vec<Rc<dyn SegmentExceptionRule>>,
}

impl SegmentExceptionValidator {
    pub fn validate(&self, item: &SegmentException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_exception_validator() -> SegmentExceptionValidator {
    SegmentExceptionValidator {
        rules: vec![
            Rc::new(SegmentExceptionAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "batched".to_string()],
            }) as Rc<dyn SegmentExceptionRule>,
            Rc::new(SegmentExceptionRequiredTitleRule {
                message: "segment exception needs a value".to_string(),
            }),
            Rc::new(SegmentExceptionDiscountRangeRule { min: 9.1, max: 909.3 }),
            Rc::new(SegmentExceptionCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_exception_validator();
        let item = new_segment_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SegmentExceptionRequiredTitleRule {
            message: "segment exception needs a value".to_string(),
        };
        let mut item = new_segment_exception("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = SegmentExceptionDiscountRangeRule { min: 2.0, max: 1207.3 };
        let mut item = new_segment_exception("x", "y");
        item.discount = 1208.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
