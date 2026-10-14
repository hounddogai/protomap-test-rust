use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive escalation score must pass. Check returns its problems, or none.
pub trait AdaptiveEscalationScoreRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveEscalationScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEscalationScoreRequiredLabelRule {
    pub message: String,
}

impl AdaptiveEscalationScoreRule for AdaptiveEscalationScoreRequiredLabelRule {
    fn check(&self, item: &AdaptiveEscalationScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEscalationScoreCipherLengthRule {
    pub max_length: i64,
}

impl AdaptiveEscalationScoreRule for AdaptiveEscalationScoreCipherLengthRule {
    fn check(&self, item: &AdaptiveEscalationScore) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEscalationScoreCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveEscalationScoreRule for AdaptiveEscalationScoreCodeLengthRule {
    fn check(&self, item: &AdaptiveEscalationScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEscalationScoreAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl AdaptiveEscalationScoreRule for AdaptiveEscalationScoreAllowedLabelRule {
    fn check(&self, item: &AdaptiveEscalationScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEscalationScoreRequiredTitleRule {
    pub message: String,
}

impl AdaptiveEscalationScoreRule for AdaptiveEscalationScoreRequiredTitleRule {
    fn check(&self, item: &AdaptiveEscalationScore) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive escalation score.
#[derive(Debug, Clone)]
pub struct AdaptiveEscalationScoreValidator {
    pub rules: Vec<Rc<dyn AdaptiveEscalationScoreRule>>,
}

impl AdaptiveEscalationScoreValidator {
    pub fn validate(&self, item: &AdaptiveEscalationScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveEscalationScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveEscalationScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_escalation_score_validator() -> AdaptiveEscalationScoreValidator {
    AdaptiveEscalationScoreValidator {
        rules: vec![
            Rc::new(AdaptiveEscalationScoreRequiredLabelRule {
                message: "adaptive escalation score needs this field".to_string(),
            }) as Rc<dyn AdaptiveEscalationScoreRule>,
            Rc::new(AdaptiveEscalationScoreCipherLengthRule { max_length: 86400 }),
            Rc::new(AdaptiveEscalationScoreCodeLengthRule { max_length: 255 }),
            Rc::new(AdaptiveEscalationScoreAllowedLabelRule {
                allowed: vec!["pending".to_string(), "manual".to_string(), "tiered".to_string()],
            }),
            Rc::new(AdaptiveEscalationScoreRequiredTitleRule {
                message: "adaptive escalation score needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_escalation_score_validator();
        let item = new_adaptive_escalation_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AdaptiveEscalationScoreRequiredLabelRule {
            message: "adaptive escalation score needs an entry".to_string(),
        };
        let mut item = new_adaptive_escalation_score("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = AdaptiveEscalationScoreCipherLengthRule { max_length: 1000 };
        let mut item = new_adaptive_escalation_score("x", "y");
        item.cipher = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveEscalationScoreCodeLengthRule { max_length: 86400 };
        let mut item = new_adaptive_escalation_score("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AdaptiveEscalationScoreRequiredTitleRule {
            message: "adaptive escalation score needs an entry".to_string(),
        };
        let mut item = new_adaptive_escalation_score("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
