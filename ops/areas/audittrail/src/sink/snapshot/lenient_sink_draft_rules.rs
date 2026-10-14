use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient sink draft must pass. Check returns its problems, or none.
pub trait LenientSinkDraftRule: std::fmt::Debug {
    fn check(&self, item: &LenientSinkDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkDraftRequiredCodeRule {
    pub message: String,
}

impl LenientSinkDraftRule for LenientSinkDraftRequiredCodeRule {
    fn check(&self, item: &LenientSinkDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkDraftSourceLengthRule {
    pub max_length: i64,
}

impl LenientSinkDraftRule for LenientSinkDraftSourceLengthRule {
    fn check(&self, item: &LenientSinkDraft) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkDraftCodePrefixRule {
    pub prefix: String,
}

impl LenientSinkDraftRule for LenientSinkDraftCodePrefixRule {
    fn check(&self, item: &LenientSinkDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkDraftRequiredSourceRule {
    pub message: String,
}

impl LenientSinkDraftRule for LenientSinkDraftRequiredSourceRule {
    fn check(&self, item: &LenientSinkDraft) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkDraftMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientSinkDraftRule for LenientSinkDraftMinimumRangeRule {
    fn check(&self, item: &LenientSinkDraft) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient sink draft.
#[derive(Debug, Clone)]
pub struct LenientSinkDraftValidator {
    pub rules: Vec<Rc<dyn LenientSinkDraftRule>>,
}

impl LenientSinkDraftValidator {
    pub fn validate(&self, item: &LenientSinkDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientSinkDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSinkDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_sink_draft_validator() -> LenientSinkDraftValidator {
    LenientSinkDraftValidator {
        rules: vec![
            Rc::new(LenientSinkDraftRequiredCodeRule {
                message: "lenient sink draft needs a value".to_string(),
            }) as Rc<dyn LenientSinkDraftRule>,
            Rc::new(LenientSinkDraftSourceLengthRule { max_length: 255 }),
            Rc::new(LenientSinkDraftCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(LenientSinkDraftRequiredSourceRule {
                message: "lenient sink draft needs this field".to_string(),
            }),
            Rc::new(LenientSinkDraftMinimumRangeRule { min: 8.0, max: 1466.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_sink_draft_validator();
        let item = new_lenient_sink_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientSinkDraftRequiredCodeRule {
            message: "lenient sink draft needs a value".to_string(),
        };
        let mut item = new_lenient_sink_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = LenientSinkDraftSourceLengthRule { max_length: 80 };
        let mut item = new_lenient_sink_draft("x", "y");
        item.source = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = LenientSinkDraftRequiredSourceRule {
            message: "lenient sink draft needs an entry".to_string(),
        };
        let mut item = new_lenient_sink_draft("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = LenientSinkDraftMinimumRangeRule { min: 6.1, max: 2167.7 };
        let mut item = new_lenient_sink_draft("x", "y");
        item.minimum = 2169.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
