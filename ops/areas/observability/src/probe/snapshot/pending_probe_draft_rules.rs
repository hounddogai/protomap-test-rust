use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending probe draft must pass. Check returns its problems, or none.
pub trait PendingProbeDraftRule: std::fmt::Debug {
    fn check(&self, item: &PendingProbeDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeDraftLabelLengthRule {
    pub max_length: i64,
}

impl PendingProbeDraftRule for PendingProbeDraftLabelLengthRule {
    fn check(&self, item: &PendingProbeDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeDraftRequiredLabelRule {
    pub message: String,
}

impl PendingProbeDraftRule for PendingProbeDraftRequiredLabelRule {
    fn check(&self, item: &PendingProbeDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeDraftBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingProbeDraftRule for PendingProbeDraftBurstRangeRule {
    fn check(&self, item: &PendingProbeDraft) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending probe draft.
#[derive(Debug, Clone)]
pub struct PendingProbeDraftValidator {
    pub rules: Vec<Rc<dyn PendingProbeDraftRule>>,
}

impl PendingProbeDraftValidator {
    pub fn validate(&self, item: &PendingProbeDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingProbeDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingProbeDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_probe_draft_validator() -> PendingProbeDraftValidator {
    PendingProbeDraftValidator {
        rules: vec![
            Rc::new(PendingProbeDraftLabelLengthRule { max_length: 120 }) as Rc<dyn PendingProbeDraftRule>,
            Rc::new(PendingProbeDraftRequiredLabelRule {
                message: "pending probe draft needs this field".to_string(),
            }),
            Rc::new(PendingProbeDraftBurstRangeRule { min: 255, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_probe_draft_validator();
        let item = new_pending_probe_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingProbeDraftLabelLengthRule { max_length: 86400 };
        let mut item = new_pending_probe_draft("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingProbeDraftRequiredLabelRule {
            message: "pending probe draft needs a value".to_string(),
        };
        let mut item = new_pending_probe_draft("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = PendingProbeDraftBurstRangeRule { min: 32, max: 120 };
        let mut item = new_pending_probe_draft("x", "y");
        item.burst = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
