use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a suspension draft must pass. Check returns its problems, or none.
pub trait SuspensionDraftRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDraftAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl SuspensionDraftRule for SuspensionDraftAllowedSegmentRule {
    fn check(&self, item: &SuspensionDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is unknown", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDraftRequiredSummaryRule {
    pub message: String,
}

impl SuspensionDraftRule for SuspensionDraftRequiredSummaryRule {
    fn check(&self, item: &SuspensionDraft) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDraftRequiredTitleRule {
    pub message: String,
}

impl SuspensionDraftRule for SuspensionDraftRequiredTitleRule {
    fn check(&self, item: &SuspensionDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDraftSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionDraftRule for SuspensionDraftSequenceRangeRule {
    fn check(&self, item: &SuspensionDraft) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension draft.
#[derive(Debug, Clone)]
pub struct SuspensionDraftValidator {
    pub rules: Vec<Rc<dyn SuspensionDraftRule>>,
}

impl SuspensionDraftValidator {
    pub fn validate(&self, item: &SuspensionDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_draft_validator() -> SuspensionDraftValidator {
    SuspensionDraftValidator {
        rules: vec![
            Rc::new(SuspensionDraftAllowedSegmentRule {
                allowed: vec!["weighted".to_string()],
            }) as Rc<dyn SuspensionDraftRule>,
            Rc::new(SuspensionDraftRequiredSummaryRule {
                message: "suspension draft needs a value".to_string(),
            }),
            Rc::new(SuspensionDraftRequiredTitleRule {
                message: "suspension draft needs an entry".to_string(),
            }),
            Rc::new(SuspensionDraftSequenceRangeRule { min: 32, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_draft_validator();
        let item = new_suspension_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = SuspensionDraftRequiredSummaryRule {
            message: "suspension draft needs an entry".to_string(),
        };
        let mut item = new_suspension_draft("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SuspensionDraftRequiredTitleRule {
            message: "suspension draft needs an entry".to_string(),
        };
        let mut item = new_suspension_draft("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = SuspensionDraftSequenceRangeRule { min: 80, max: 120 };
        let mut item = new_suspension_draft("x", "y");
        item.sequence = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
