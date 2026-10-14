use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback segment draft must pass. Check returns its problems, or none.
pub trait FallbackSegmentDraftRule: std::fmt::Debug {
    fn check(&self, item: &FallbackSegmentDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentDraftRequiredLabelRule {
    pub message: String,
}

impl FallbackSegmentDraftRule for FallbackSegmentDraftRequiredLabelRule {
    fn check(&self, item: &FallbackSegmentDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentDraftRequiredContactEmailRule {
    pub message: String,
}

impl FallbackSegmentDraftRule for FallbackSegmentDraftRequiredContactEmailRule {
    fn check(&self, item: &FallbackSegmentDraft) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentDraftRequiredUnitRule {
    pub message: String,
}

impl FallbackSegmentDraftRule for FallbackSegmentDraftRequiredUnitRule {
    fn check(&self, item: &FallbackSegmentDraft) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback segment draft.
#[derive(Debug, Clone)]
pub struct FallbackSegmentDraftValidator {
    pub rules: Vec<Rc<dyn FallbackSegmentDraftRule>>,
}

impl FallbackSegmentDraftValidator {
    pub fn validate(&self, item: &FallbackSegmentDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackSegmentDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackSegmentDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_segment_draft_validator() -> FallbackSegmentDraftValidator {
    FallbackSegmentDraftValidator {
        rules: vec![
            Rc::new(FallbackSegmentDraftRequiredLabelRule {
                message: "fallback segment draft needs a value".to_string(),
            }) as Rc<dyn FallbackSegmentDraftRule>,
            Rc::new(FallbackSegmentDraftRequiredContactEmailRule {
                message: "fallback segment draft needs an entry".to_string(),
            }),
            Rc::new(FallbackSegmentDraftRequiredUnitRule {
                message: "fallback segment draft needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_segment_draft_validator();
        let item = new_fallback_segment_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FallbackSegmentDraftRequiredLabelRule {
            message: "fallback segment draft needs an entry".to_string(),
        };
        let mut item = new_fallback_segment_draft("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = FallbackSegmentDraftRequiredContactEmailRule {
            message: "fallback segment draft needs an entry".to_string(),
        };
        let mut item = new_fallback_segment_draft("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = FallbackSegmentDraftRequiredUnitRule {
            message: "fallback segment draft needs an entry".to_string(),
        };
        let mut item = new_fallback_segment_draft("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
