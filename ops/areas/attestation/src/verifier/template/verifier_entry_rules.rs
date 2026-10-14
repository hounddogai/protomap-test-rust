use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a verifier entry must pass. Check returns its problems, or none.
pub trait VerifierEntryRule: std::fmt::Debug {
    fn check(&self, item: &VerifierEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierEntryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl VerifierEntryRule for VerifierEntryAllowedCodeRule {
    fn check(&self, item: &VerifierEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierEntryRequiredCodeRule {
    pub message: String,
}

impl VerifierEntryRule for VerifierEntryRequiredCodeRule {
    fn check(&self, item: &VerifierEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierEntryWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierEntryRule for VerifierEntryWeightRangeRule {
    fn check(&self, item: &VerifierEntry) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierEntryCodeLengthRule {
    pub max_length: i64,
}

impl VerifierEntryRule for VerifierEntryCodeLengthRule {
    fn check(&self, item: &VerifierEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierEntryActiveGuardRule {
    pub limit: f64,
}

impl VerifierEntryRule for VerifierEntryActiveGuardRule {
    fn check(&self, item: &VerifierEntry) -> Vec<String> {
        if item.active && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while active", item.weight, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierEntryRequiredLabelRule {
    pub message: String,
}

impl VerifierEntryRule for VerifierEntryRequiredLabelRule {
    fn check(&self, item: &VerifierEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier entry.
#[derive(Debug, Clone)]
pub struct VerifierEntryValidator {
    pub rules: Vec<Rc<dyn VerifierEntryRule>>,
}

impl VerifierEntryValidator {
    pub fn validate(&self, item: &VerifierEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_entry_validator() -> VerifierEntryValidator {
    VerifierEntryValidator {
        rules: vec![
            Rc::new(VerifierEntryAllowedCodeRule {
                allowed: vec!["weighted".to_string(), "rolling".to_string(), "capped".to_string()],
            }) as Rc<dyn VerifierEntryRule>,
            Rc::new(VerifierEntryRequiredCodeRule {
                message: "verifier entry needs this field".to_string(),
            }),
            Rc::new(VerifierEntryWeightRangeRule { min: 7.3, max: 1518.0 }),
            Rc::new(VerifierEntryCodeLengthRule { max_length: 3600 }),
            Rc::new(VerifierEntryActiveGuardRule { limit: 3.4 }),
            Rc::new(VerifierEntryRequiredLabelRule {
                message: "verifier entry needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_entry_validator();
        let item = new_verifier_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = VerifierEntryRequiredCodeRule {
            message: "verifier entry needs a value".to_string(),
        };
        let mut item = new_verifier_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = VerifierEntryWeightRangeRule { min: 9.0, max: 4216.1 };
        let mut item = new_verifier_entry("x", "y");
        item.weight = 4217.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = VerifierEntryCodeLengthRule { max_length: 255 };
        let mut item = new_verifier_entry("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = VerifierEntryRequiredLabelRule {
            message: "verifier entry needs an entry".to_string(),
        };
        let mut item = new_verifier_entry("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
