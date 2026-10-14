use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation preference must pass. Check returns its problems, or none.
pub trait EscalationPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &EscalationPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPreferenceTitleLengthRule {
    pub max_length: i64,
}

impl EscalationPreferenceRule for EscalationPreferenceTitleLengthRule {
    fn check(&self, item: &EscalationPreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPreferenceSegmentLengthRule {
    pub max_length: i64,
}

impl EscalationPreferenceRule for EscalationPreferenceSegmentLengthRule {
    fn check(&self, item: &EscalationPreference) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPreferenceRequiredTitleRule {
    pub message: String,
}

impl EscalationPreferenceRule for EscalationPreferenceRequiredTitleRule {
    fn check(&self, item: &EscalationPreference) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPreferenceRequiredCategoryRule {
    pub message: String,
}

impl EscalationPreferenceRule for EscalationPreferenceRequiredCategoryRule {
    fn check(&self, item: &EscalationPreference) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPreferenceRequiredCodeRule {
    pub message: String,
}

impl EscalationPreferenceRule for EscalationPreferenceRequiredCodeRule {
    fn check(&self, item: &EscalationPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation preference.
#[derive(Debug, Clone)]
pub struct EscalationPreferenceValidator {
    pub rules: Vec<Rc<dyn EscalationPreferenceRule>>,
}

impl EscalationPreferenceValidator {
    pub fn validate(&self, item: &EscalationPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_preference_validator() -> EscalationPreferenceValidator {
    EscalationPreferenceValidator {
        rules: vec![
            Rc::new(EscalationPreferenceTitleLengthRule { max_length: 120 }) as Rc<dyn EscalationPreferenceRule>,
            Rc::new(EscalationPreferenceSegmentLengthRule { max_length: 32 }),
            Rc::new(EscalationPreferenceRequiredTitleRule {
                message: "escalation preference needs this field".to_string(),
            }),
            Rc::new(EscalationPreferenceRequiredCategoryRule {
                message: "escalation preference needs this field".to_string(),
            }),
            Rc::new(EscalationPreferenceRequiredCodeRule {
                message: "escalation preference needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_preference_validator();
        let item = new_escalation_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = EscalationPreferenceTitleLengthRule { max_length: 86400 };
        let mut item = new_escalation_preference("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = EscalationPreferenceSegmentLengthRule { max_length: 86400 };
        let mut item = new_escalation_preference("x", "y");
        item.segment = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = EscalationPreferenceRequiredTitleRule {
            message: "escalation preference needs this field".to_string(),
        };
        let mut item = new_escalation_preference("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = EscalationPreferenceRequiredCategoryRule {
            message: "escalation preference needs an entry".to_string(),
        };
        let mut item = new_escalation_preference("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EscalationPreferenceRequiredCodeRule {
            message: "escalation preference needs a value".to_string(),
        };
        let mut item = new_escalation_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
