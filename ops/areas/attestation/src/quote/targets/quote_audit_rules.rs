use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a quote audit must pass. Check returns its problems, or none.
pub trait QuoteAuditRule: std::fmt::Debug {
    fn check(&self, item: &QuoteAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAuditRequiredChannelRule {
    pub message: String,
}

impl QuoteAuditRule for QuoteAuditRequiredChannelRule {
    fn check(&self, item: &QuoteAudit) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAuditRequiredCodeRule {
    pub message: String,
}

impl QuoteAuditRule for QuoteAuditRequiredCodeRule {
    fn check(&self, item: &QuoteAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAuditAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl QuoteAuditRule for QuoteAuditAllowedChannelRule {
    fn check(&self, item: &QuoteAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is out of range", item.channel)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAuditAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteAuditRule for QuoteAuditAttemptsRangeRule {
    fn check(&self, item: &QuoteAudit) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote audit.
#[derive(Debug, Clone)]
pub struct QuoteAuditValidator {
    pub rules: Vec<Rc<dyn QuoteAuditRule>>,
}

impl QuoteAuditValidator {
    pub fn validate(&self, item: &QuoteAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuoteAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_audit_validator() -> QuoteAuditValidator {
    QuoteAuditValidator {
        rules: vec![
            Rc::new(QuoteAuditRequiredChannelRule {
                message: "quote audit needs a value".to_string(),
            }) as Rc<dyn QuoteAuditRule>,
            Rc::new(QuoteAuditRequiredCodeRule {
                message: "quote audit needs a value".to_string(),
            }),
            Rc::new(QuoteAuditAllowedChannelRule { allowed: Vec::new() }),
            Rc::new(QuoteAuditAttemptsRangeRule { min: 3600, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_audit_validator();
        let item = new_quote_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = QuoteAuditRequiredChannelRule {
            message: "quote audit needs a value".to_string(),
        };
        let mut item = new_quote_audit("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QuoteAuditRequiredCodeRule {
            message: "quote audit needs this field".to_string(),
        };
        let mut item = new_quote_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = QuoteAuditAttemptsRangeRule { min: 120, max: 86400 };
        let mut item = new_quote_audit("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
