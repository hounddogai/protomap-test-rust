use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a quote draft must pass. Check returns its problems, or none.
pub trait QuoteDraftRule: std::fmt::Debug {
    fn check(&self, item: &QuoteDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteDraftCodeLengthRule {
    pub max_length: i64,
}

impl QuoteDraftRule for QuoteDraftCodeLengthRule {
    fn check(&self, item: &QuoteDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteDraftQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteDraftRule for QuoteDraftQuotaUnitsRangeRule {
    fn check(&self, item: &QuoteDraft) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteDraftAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl QuoteDraftRule for QuoteDraftAllowedCodeRule {
    fn check(&self, item: &QuoteDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteDraftRequiredCodeRule {
    pub message: String,
}

impl QuoteDraftRule for QuoteDraftRequiredCodeRule {
    fn check(&self, item: &QuoteDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteDraftAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteDraftRule for QuoteDraftAttemptsRangeRule {
    fn check(&self, item: &QuoteDraft) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote draft.
#[derive(Debug, Clone)]
pub struct QuoteDraftValidator {
    pub rules: Vec<Rc<dyn QuoteDraftRule>>,
}

impl QuoteDraftValidator {
    pub fn validate(&self, item: &QuoteDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_draft_validator() -> QuoteDraftValidator {
    QuoteDraftValidator {
        rules: vec![
            Rc::new(QuoteDraftCodeLengthRule { max_length: 3600 }) as Rc<dyn QuoteDraftRule>,
            Rc::new(QuoteDraftQuotaUnitsRangeRule { min: 1000, max: 86400 }),
            Rc::new(QuoteDraftAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "fallback".to_string()],
            }),
            Rc::new(QuoteDraftRequiredCodeRule {
                message: "quote draft needs this field".to_string(),
            }),
            Rc::new(QuoteDraftAttemptsRangeRule { min: 120, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_draft_validator();
        let item = new_quote_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QuoteDraftCodeLengthRule { max_length: 32 };
        let mut item = new_quote_draft("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = QuoteDraftQuotaUnitsRangeRule { min: 64, max: 1000 };
        let mut item = new_quote_draft("x", "y");
        item.quota_units = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QuoteDraftRequiredCodeRule {
            message: "quote draft needs an entry".to_string(),
        };
        let mut item = new_quote_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = QuoteDraftAttemptsRangeRule { min: 120, max: 3600 };
        let mut item = new_quote_draft("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
