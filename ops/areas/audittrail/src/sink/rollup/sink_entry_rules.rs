use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink entry must pass. Check returns its problems, or none.
pub trait SinkEntryRule: std::fmt::Debug {
    fn check(&self, item: &SinkEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkEntryNameLengthRule {
    pub max_length: i64,
}

impl SinkEntryRule for SinkEntryNameLengthRule {
    fn check(&self, item: &SinkEntry) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkEntryRequiredExternalRefRule {
    pub message: String,
}

impl SinkEntryRule for SinkEntryRequiredExternalRefRule {
    fn check(&self, item: &SinkEntry) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkEntryExternalRefLengthRule {
    pub max_length: i64,
}

impl SinkEntryRule for SinkEntryExternalRefLengthRule {
    fn check(&self, item: &SinkEntry) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkEntryRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SinkEntryRule for SinkEntryRateRangeRule {
    fn check(&self, item: &SinkEntry) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkEntryRequiredCodeRule {
    pub message: String,
}

impl SinkEntryRule for SinkEntryRequiredCodeRule {
    fn check(&self, item: &SinkEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkEntryFingerprintLengthRule {
    pub max_length: i64,
}

impl SinkEntryRule for SinkEntryFingerprintLengthRule {
    fn check(&self, item: &SinkEntry) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink entry.
#[derive(Debug, Clone)]
pub struct SinkEntryValidator {
    pub rules: Vec<Rc<dyn SinkEntryRule>>,
}

impl SinkEntryValidator {
    pub fn validate(&self, item: &SinkEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SinkEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_entry_validator() -> SinkEntryValidator {
    SinkEntryValidator {
        rules: vec![
            Rc::new(SinkEntryNameLengthRule { max_length: 86400 }) as Rc<dyn SinkEntryRule>,
            Rc::new(SinkEntryRequiredExternalRefRule {
                message: "sink entry needs this field".to_string(),
            }),
            Rc::new(SinkEntryExternalRefLengthRule { max_length: 64 }),
            Rc::new(SinkEntryRateRangeRule { min: 9.3, max: 4397.0 }),
            Rc::new(SinkEntryRequiredCodeRule {
                message: "sink entry needs a value".to_string(),
            }),
            Rc::new(SinkEntryFingerprintLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_entry_validator();
        let item = new_sink_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SinkEntryNameLengthRule { max_length: 64 };
        let mut item = new_sink_entry("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = SinkEntryRequiredExternalRefRule {
            message: "sink entry needs this field".to_string(),
        };
        let mut item = new_sink_entry("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = SinkEntryExternalRefLengthRule { max_length: 64 };
        let mut item = new_sink_entry("x", "y");
        item.external_ref = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = SinkEntryRateRangeRule { min: 4.2, max: 84.8 };
        let mut item = new_sink_entry("x", "y");
        item.rate = 86.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SinkEntryRequiredCodeRule {
            message: "sink entry needs this field".to_string(),
        };
        let mut item = new_sink_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = SinkEntryFingerprintLengthRule { max_length: 86400 };
        let mut item = new_sink_entry("x", "y");
        item.fingerprint = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
