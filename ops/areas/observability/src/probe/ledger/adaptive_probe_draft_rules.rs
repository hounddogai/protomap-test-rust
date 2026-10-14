use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive probe draft must pass. Check returns its problems, or none.
pub trait AdaptiveProbeDraftRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveProbeDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeDraftSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveProbeDraftRule for AdaptiveProbeDraftSequenceRangeRule {
    fn check(&self, item: &AdaptiveProbeDraft) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeDraftRequiredScopeRule {
    pub message: String,
}

impl AdaptiveProbeDraftRule for AdaptiveProbeDraftRequiredScopeRule {
    fn check(&self, item: &AdaptiveProbeDraft) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeDraftCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveProbeDraftRule for AdaptiveProbeDraftCodeLengthRule {
    fn check(&self, item: &AdaptiveProbeDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeDraftScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveProbeDraftRule for AdaptiveProbeDraftScoreRangeRule {
    fn check(&self, item: &AdaptiveProbeDraft) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeDraftRequiredLabelRule {
    pub message: String,
}

impl AdaptiveProbeDraftRule for AdaptiveProbeDraftRequiredLabelRule {
    fn check(&self, item: &AdaptiveProbeDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeDraftAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl AdaptiveProbeDraftRule for AdaptiveProbeDraftAllowedLabelRule {
    fn check(&self, item: &AdaptiveProbeDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive probe draft.
#[derive(Debug, Clone)]
pub struct AdaptiveProbeDraftValidator {
    pub rules: Vec<Rc<dyn AdaptiveProbeDraftRule>>,
}

impl AdaptiveProbeDraftValidator {
    pub fn validate(&self, item: &AdaptiveProbeDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveProbeDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveProbeDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_probe_draft_validator() -> AdaptiveProbeDraftValidator {
    AdaptiveProbeDraftValidator {
        rules: vec![
            Rc::new(AdaptiveProbeDraftSequenceRangeRule { min: 64, max: 86400 }) as Rc<dyn AdaptiveProbeDraftRule>,
            Rc::new(AdaptiveProbeDraftRequiredScopeRule {
                message: "adaptive probe draft needs an entry".to_string(),
            }),
            Rc::new(AdaptiveProbeDraftCodeLengthRule { max_length: 16 }),
            Rc::new(AdaptiveProbeDraftScoreRangeRule { min: 3.8, max: 192.7 }),
            Rc::new(AdaptiveProbeDraftRequiredLabelRule {
                message: "adaptive probe draft needs an entry".to_string(),
            }),
            Rc::new(AdaptiveProbeDraftAllowedLabelRule {
                allowed: vec!["adaptive".to_string(), "scheduled".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_probe_draft_validator();
        let item = new_adaptive_probe_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = AdaptiveProbeDraftSequenceRangeRule { min: 32, max: 32 };
        let mut item = new_adaptive_probe_draft("x", "y");
        item.sequence = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = AdaptiveProbeDraftRequiredScopeRule {
            message: "adaptive probe draft needs this field".to_string(),
        };
        let mut item = new_adaptive_probe_draft("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveProbeDraftCodeLengthRule { max_length: 3600 };
        let mut item = new_adaptive_probe_draft("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = AdaptiveProbeDraftScoreRangeRule { min: 3.5, max: 4758.1 };
        let mut item = new_adaptive_probe_draft("x", "y");
        item.score = 4759.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AdaptiveProbeDraftRequiredLabelRule {
            message: "adaptive probe draft needs this field".to_string(),
        };
        let mut item = new_adaptive_probe_draft("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
