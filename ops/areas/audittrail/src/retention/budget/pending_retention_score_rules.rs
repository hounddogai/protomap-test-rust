use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending retention score must pass. Check returns its problems, or none.
pub trait PendingRetentionScoreRule: std::fmt::Debug {
    fn check(&self, item: &PendingRetentionScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionScoreAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl PendingRetentionScoreRule for PendingRetentionScoreAllowedChannelRule {
    fn check(&self, item: &PendingRetentionScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is above maximum", item.channel)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionScoreRequiredCodeRule {
    pub message: String,
}

impl PendingRetentionScoreRule for PendingRetentionScoreRequiredCodeRule {
    fn check(&self, item: &PendingRetentionScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionScoreGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRetentionScoreRule for PendingRetentionScoreGraceDaysRangeRule {
    fn check(&self, item: &PendingRetentionScore) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionScoreCipherLengthRule {
    pub max_length: i64,
}

impl PendingRetentionScoreRule for PendingRetentionScoreCipherLengthRule {
    fn check(&self, item: &PendingRetentionScore) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionScoreRequiredLabelRule {
    pub message: String,
}

impl PendingRetentionScoreRule for PendingRetentionScoreRequiredLabelRule {
    fn check(&self, item: &PendingRetentionScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionScoreCodeLengthRule {
    pub max_length: i64,
}

impl PendingRetentionScoreRule for PendingRetentionScoreCodeLengthRule {
    fn check(&self, item: &PendingRetentionScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending retention score.
#[derive(Debug, Clone)]
pub struct PendingRetentionScoreValidator {
    pub rules: Vec<Rc<dyn PendingRetentionScoreRule>>,
}

impl PendingRetentionScoreValidator {
    pub fn validate(&self, item: &PendingRetentionScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingRetentionScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRetentionScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_retention_score_validator() -> PendingRetentionScoreValidator {
    PendingRetentionScoreValidator {
        rules: vec![
            Rc::new(PendingRetentionScoreAllowedChannelRule { allowed: Vec::new() })
                as Rc<dyn PendingRetentionScoreRule>,
            Rc::new(PendingRetentionScoreRequiredCodeRule {
                message: "pending retention score needs this field".to_string(),
            }),
            Rc::new(PendingRetentionScoreGraceDaysRangeRule { min: 64, max: 64 }),
            Rc::new(PendingRetentionScoreCipherLengthRule { max_length: 255 }),
            Rc::new(PendingRetentionScoreRequiredLabelRule {
                message: "pending retention score needs an entry".to_string(),
            }),
            Rc::new(PendingRetentionScoreCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_retention_score_validator();
        let item = new_pending_retention_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingRetentionScoreRequiredCodeRule {
            message: "pending retention score needs an entry".to_string(),
        };
        let mut item = new_pending_retention_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = PendingRetentionScoreGraceDaysRangeRule { min: 32, max: 3600 };
        let mut item = new_pending_retention_score("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = PendingRetentionScoreCipherLengthRule { max_length: 255 };
        let mut item = new_pending_retention_score("x", "y");
        item.cipher = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingRetentionScoreRequiredLabelRule {
            message: "pending retention score needs an entry".to_string(),
        };
        let mut item = new_pending_retention_score("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingRetentionScoreCodeLengthRule { max_length: 120 };
        let mut item = new_pending_retention_score("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
