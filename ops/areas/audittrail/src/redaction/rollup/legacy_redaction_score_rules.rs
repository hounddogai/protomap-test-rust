use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy redaction score must pass. Check returns its problems, or none.
pub trait LegacyRedactionScoreRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRedactionScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionScoreLabelLengthRule {
    pub max_length: i64,
}

impl LegacyRedactionScoreRule for LegacyRedactionScoreLabelLengthRule {
    fn check(&self, item: &LegacyRedactionScore) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionScoreAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LegacyRedactionScoreRule for LegacyRedactionScoreAllowedCodeRule {
    fn check(&self, item: &LegacyRedactionScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionScoreRequiredCodeRule {
    pub message: String,
}

impl LegacyRedactionScoreRule for LegacyRedactionScoreRequiredCodeRule {
    fn check(&self, item: &LegacyRedactionScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionScoreThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyRedactionScoreRule for LegacyRedactionScoreThresholdRangeRule {
    fn check(&self, item: &LegacyRedactionScore) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionScoreRequiredSummaryRule {
    pub message: String,
}

impl LegacyRedactionScoreRule for LegacyRedactionScoreRequiredSummaryRule {
    fn check(&self, item: &LegacyRedactionScore) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionScoreCodePrefixRule {
    pub prefix: String,
}

impl LegacyRedactionScoreRule for LegacyRedactionScoreCodePrefixRule {
    fn check(&self, item: &LegacyRedactionScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy redaction score.
#[derive(Debug, Clone)]
pub struct LegacyRedactionScoreValidator {
    pub rules: Vec<Rc<dyn LegacyRedactionScoreRule>>,
}

impl LegacyRedactionScoreValidator {
    pub fn validate(&self, item: &LegacyRedactionScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyRedactionScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRedactionScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_redaction_score_validator() -> LegacyRedactionScoreValidator {
    LegacyRedactionScoreValidator {
        rules: vec![
            Rc::new(LegacyRedactionScoreLabelLengthRule { max_length: 1000 }) as Rc<dyn LegacyRedactionScoreRule>,
            Rc::new(LegacyRedactionScoreAllowedCodeRule {
                allowed: vec!["regional".to_string()],
            }),
            Rc::new(LegacyRedactionScoreRequiredCodeRule {
                message: "legacy redaction score needs an entry".to_string(),
            }),
            Rc::new(LegacyRedactionScoreThresholdRangeRule { min: 6.8, max: 146.4 }),
            Rc::new(LegacyRedactionScoreRequiredSummaryRule {
                message: "legacy redaction score needs an entry".to_string(),
            }),
            Rc::new(LegacyRedactionScoreCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_redaction_score_validator();
        let item = new_legacy_redaction_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacyRedactionScoreLabelLengthRule { max_length: 86400 };
        let mut item = new_legacy_redaction_score("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyRedactionScoreRequiredCodeRule {
            message: "legacy redaction score needs a value".to_string(),
        };
        let mut item = new_legacy_redaction_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = LegacyRedactionScoreThresholdRangeRule { min: 0.9, max: 4395.5 };
        let mut item = new_legacy_redaction_score("x", "y");
        item.threshold = 4397.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = LegacyRedactionScoreRequiredSummaryRule {
            message: "legacy redaction score needs an entry".to_string(),
        };
        let mut item = new_legacy_redaction_score("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
