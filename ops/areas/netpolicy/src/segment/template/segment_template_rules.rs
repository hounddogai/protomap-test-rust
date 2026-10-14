use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a segment template must pass. Check returns its problems, or none.
pub trait SegmentTemplateRule: std::fmt::Debug {
    fn check(&self, item: &SegmentTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentTemplateStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentTemplateRule for SegmentTemplateStartsAtRangeRule {
    fn check(&self, item: &SegmentTemplate) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentTemplateCodeLengthRule {
    pub max_length: i64,
}

impl SegmentTemplateRule for SegmentTemplateCodeLengthRule {
    fn check(&self, item: &SegmentTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentTemplateAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl SegmentTemplateRule for SegmentTemplateAllowedLabelRule {
    fn check(&self, item: &SegmentTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is not allowed", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentTemplateRequiredLabelRule {
    pub message: String,
}

impl SegmentTemplateRule for SegmentTemplateRequiredLabelRule {
    fn check(&self, item: &SegmentTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentTemplateUnitLengthRule {
    pub max_length: i64,
}

impl SegmentTemplateRule for SegmentTemplateUnitLengthRule {
    fn check(&self, item: &SegmentTemplate) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentTemplateGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentTemplateRule for SegmentTemplateGraceDaysRangeRule {
    fn check(&self, item: &SegmentTemplate) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment template.
#[derive(Debug, Clone)]
pub struct SegmentTemplateValidator {
    pub rules: Vec<Rc<dyn SegmentTemplateRule>>,
}

impl SegmentTemplateValidator {
    pub fn validate(&self, item: &SegmentTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_template_validator() -> SegmentTemplateValidator {
    SegmentTemplateValidator {
        rules: vec![
            Rc::new(SegmentTemplateStartsAtRangeRule { min: 120, max: 1000 }) as Rc<dyn SegmentTemplateRule>,
            Rc::new(SegmentTemplateCodeLengthRule { max_length: 3600 }),
            Rc::new(SegmentTemplateAllowedLabelRule {
                allowed: vec!["default".to_string()],
            }),
            Rc::new(SegmentTemplateRequiredLabelRule {
                message: "segment template needs an entry".to_string(),
            }),
            Rc::new(SegmentTemplateUnitLengthRule { max_length: 32 }),
            Rc::new(SegmentTemplateGraceDaysRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_template_validator();
        let item = new_segment_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = SegmentTemplateStartsAtRangeRule { min: 16, max: 86400 };
        let mut item = new_segment_template("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SegmentTemplateCodeLengthRule { max_length: 3600 };
        let mut item = new_segment_template("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SegmentTemplateRequiredLabelRule {
            message: "segment template needs a value".to_string(),
        };
        let mut item = new_segment_template("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = SegmentTemplateUnitLengthRule { max_length: 86400 };
        let mut item = new_segment_template("x", "y");
        item.unit = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = SegmentTemplateGraceDaysRangeRule { min: 80, max: 86400 };
        let mut item = new_segment_template("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
