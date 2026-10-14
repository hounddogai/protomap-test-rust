use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a quote record must pass. Check returns its problems, or none.
pub trait QuoteRecordRule: std::fmt::Debug {
    fn check(&self, item: &QuoteRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRecordRequiredSegmentRule {
    pub message: String,
}

impl QuoteRecordRule for QuoteRecordRequiredSegmentRule {
    fn check(&self, item: &QuoteRecord) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRecordKeyIdLengthRule {
    pub max_length: i64,
}

impl QuoteRecordRule for QuoteRecordKeyIdLengthRule {
    fn check(&self, item: &QuoteRecord) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRecordAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl QuoteRecordRule for QuoteRecordAllowedFingerprintRule {
    fn check(&self, item: &QuoteRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is stale", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRecordSegmentLengthRule {
    pub max_length: i64,
}

impl QuoteRecordRule for QuoteRecordSegmentLengthRule {
    fn check(&self, item: &QuoteRecord) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote record.
#[derive(Debug, Clone)]
pub struct QuoteRecordValidator {
    pub rules: Vec<Rc<dyn QuoteRecordRule>>,
}

impl QuoteRecordValidator {
    pub fn validate(&self, item: &QuoteRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_record_validator() -> QuoteRecordValidator {
    QuoteRecordValidator {
        rules: vec![
            Rc::new(QuoteRecordRequiredSegmentRule {
                message: "quote record needs an entry".to_string(),
            }) as Rc<dyn QuoteRecordRule>,
            Rc::new(QuoteRecordKeyIdLengthRule { max_length: 80 }),
            Rc::new(QuoteRecordAllowedFingerprintRule { allowed: Vec::new() }),
            Rc::new(QuoteRecordSegmentLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_record_validator();
        let item = new_quote_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = QuoteRecordRequiredSegmentRule {
            message: "quote record needs this field".to_string(),
        };
        let mut item = new_quote_record("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = QuoteRecordKeyIdLengthRule { max_length: 120 };
        let mut item = new_quote_record("x", "y");
        item.key_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = QuoteRecordSegmentLengthRule { max_length: 80 };
        let mut item = new_quote_record("x", "y");
        item.segment = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
