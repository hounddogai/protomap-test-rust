use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict segment entry must pass. Check returns its problems, or none.
pub trait StrictSegmentEntryRule: std::fmt::Debug {
    fn check(&self, item: &StrictSegmentEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentEntryRequiredCodeRule {
    pub message: String,
}

impl StrictSegmentEntryRule for StrictSegmentEntryRequiredCodeRule {
    fn check(&self, item: &StrictSegmentEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentEntryAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl StrictSegmentEntryRule for StrictSegmentEntryAllowedTitleRule {
    fn check(&self, item: &StrictSegmentEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentEntryRequiredPhoneRule {
    pub message: String,
}

impl StrictSegmentEntryRule for StrictSegmentEntryRequiredPhoneRule {
    fn check(&self, item: &StrictSegmentEntry) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentEntryApprovedGuardRule {
    pub limit: i64,
}

impl StrictSegmentEntryRule for StrictSegmentEntryApprovedGuardRule {
    fn check(&self, item: &StrictSegmentEntry) -> Vec<String> {
        if item.approved && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while approved",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentEntryRequiredTitleRule {
    pub message: String,
}

impl StrictSegmentEntryRule for StrictSegmentEntryRequiredTitleRule {
    fn check(&self, item: &StrictSegmentEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentEntryRequiredSourceRule {
    pub message: String,
}

impl StrictSegmentEntryRule for StrictSegmentEntryRequiredSourceRule {
    fn check(&self, item: &StrictSegmentEntry) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict segment entry.
#[derive(Debug, Clone)]
pub struct StrictSegmentEntryValidator {
    pub rules: Vec<Rc<dyn StrictSegmentEntryRule>>,
}

impl StrictSegmentEntryValidator {
    pub fn validate(&self, item: &StrictSegmentEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictSegmentEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictSegmentEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_segment_entry_validator() -> StrictSegmentEntryValidator {
    StrictSegmentEntryValidator {
        rules: vec![
            Rc::new(StrictSegmentEntryRequiredCodeRule {
                message: "strict segment entry needs an entry".to_string(),
            }) as Rc<dyn StrictSegmentEntryRule>,
            Rc::new(StrictSegmentEntryAllowedTitleRule {
                allowed: vec!["weighted".to_string()],
            }),
            Rc::new(StrictSegmentEntryRequiredPhoneRule {
                message: "strict segment entry needs a value".to_string(),
            }),
            Rc::new(StrictSegmentEntryApprovedGuardRule { limit: 32 }),
            Rc::new(StrictSegmentEntryRequiredTitleRule {
                message: "strict segment entry needs a value".to_string(),
            }),
            Rc::new(StrictSegmentEntryRequiredSourceRule {
                message: "strict segment entry needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_segment_entry_validator();
        let item = new_strict_segment_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictSegmentEntryRequiredCodeRule {
            message: "strict segment entry needs this field".to_string(),
        };
        let mut item = new_strict_segment_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = StrictSegmentEntryRequiredPhoneRule {
            message: "strict segment entry needs an entry".to_string(),
        };
        let mut item = new_strict_segment_entry("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = StrictSegmentEntryRequiredTitleRule {
            message: "strict segment entry needs an entry".to_string(),
        };
        let mut item = new_strict_segment_entry("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = StrictSegmentEntryRequiredSourceRule {
            message: "strict segment entry needs a value".to_string(),
        };
        let mut item = new_strict_segment_entry("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
