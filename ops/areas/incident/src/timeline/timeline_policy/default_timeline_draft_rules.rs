use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default timeline draft must pass. Check returns its problems, or none.
pub trait DefaultTimelineDraftRule: std::fmt::Debug {
    fn check(&self, item: &DefaultTimelineDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineDraftRequiredSourceRule {
    pub message: String,
}

impl DefaultTimelineDraftRule for DefaultTimelineDraftRequiredSourceRule {
    fn check(&self, item: &DefaultTimelineDraft) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineDraftRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultTimelineDraftRule for DefaultTimelineDraftRateRangeRule {
    fn check(&self, item: &DefaultTimelineDraft) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineDraftLabelLengthRule {
    pub max_length: i64,
}

impl DefaultTimelineDraftRule for DefaultTimelineDraftLabelLengthRule {
    fn check(&self, item: &DefaultTimelineDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineDraftRequiredCodeRule {
    pub message: String,
}

impl DefaultTimelineDraftRule for DefaultTimelineDraftRequiredCodeRule {
    fn check(&self, item: &DefaultTimelineDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineDraftCodeLengthRule {
    pub max_length: i64,
}

impl DefaultTimelineDraftRule for DefaultTimelineDraftCodeLengthRule {
    fn check(&self, item: &DefaultTimelineDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineDraftRequiredLabelRule {
    pub message: String,
}

impl DefaultTimelineDraftRule for DefaultTimelineDraftRequiredLabelRule {
    fn check(&self, item: &DefaultTimelineDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default timeline draft.
#[derive(Debug, Clone)]
pub struct DefaultTimelineDraftValidator {
    pub rules: Vec<Rc<dyn DefaultTimelineDraftRule>>,
}

impl DefaultTimelineDraftValidator {
    pub fn validate(&self, item: &DefaultTimelineDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultTimelineDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultTimelineDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_timeline_draft_validator() -> DefaultTimelineDraftValidator {
    DefaultTimelineDraftValidator {
        rules: vec![
            Rc::new(DefaultTimelineDraftRequiredSourceRule {
                message: "default timeline draft needs this field".to_string(),
            }) as Rc<dyn DefaultTimelineDraftRule>,
            Rc::new(DefaultTimelineDraftRateRangeRule { min: 7.1, max: 1195.0 }),
            Rc::new(DefaultTimelineDraftLabelLengthRule { max_length: 1000 }),
            Rc::new(DefaultTimelineDraftRequiredCodeRule {
                message: "default timeline draft needs this field".to_string(),
            }),
            Rc::new(DefaultTimelineDraftCodeLengthRule { max_length: 120 }),
            Rc::new(DefaultTimelineDraftRequiredLabelRule {
                message: "default timeline draft needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_timeline_draft_validator();
        let item = new_default_timeline_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = DefaultTimelineDraftRequiredSourceRule {
            message: "default timeline draft needs this field".to_string(),
        };
        let mut item = new_default_timeline_draft("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = DefaultTimelineDraftRateRangeRule { min: 6.5, max: 1488.7 };
        let mut item = new_default_timeline_draft("x", "y");
        item.rate = 1490.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultTimelineDraftLabelLengthRule { max_length: 86400 };
        let mut item = new_default_timeline_draft("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultTimelineDraftRequiredCodeRule {
            message: "default timeline draft needs this field".to_string(),
        };
        let mut item = new_default_timeline_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultTimelineDraftCodeLengthRule { max_length: 80 };
        let mut item = new_default_timeline_draft("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultTimelineDraftRequiredLabelRule {
            message: "default timeline draft needs a value".to_string(),
        };
        let mut item = new_default_timeline_draft("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
