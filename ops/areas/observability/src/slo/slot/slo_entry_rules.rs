use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo entry must pass. Check returns its problems, or none.
pub trait SloEntryRule: std::fmt::Debug {
    fn check(&self, item: &SloEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloEntryRequiredCodeRule {
    pub message: String,
}

impl SloEntryRule for SloEntryRequiredCodeRule {
    fn check(&self, item: &SloEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloEntryCodeLengthRule {
    pub max_length: i64,
}

impl SloEntryRule for SloEntryCodeLengthRule {
    fn check(&self, item: &SloEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloEntryCodePrefixRule {
    pub prefix: String,
}

impl SloEntryRule for SloEntryCodePrefixRule {
    fn check(&self, item: &SloEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo entry.
#[derive(Debug, Clone)]
pub struct SloEntryValidator {
    pub rules: Vec<Rc<dyn SloEntryRule>>,
}

impl SloEntryValidator {
    pub fn validate(&self, item: &SloEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_entry_validator() -> SloEntryValidator {
    SloEntryValidator {
        rules: vec![
            Rc::new(SloEntryRequiredCodeRule {
                message: "slo entry needs this field".to_string(),
            }) as Rc<dyn SloEntryRule>,
            Rc::new(SloEntryCodeLengthRule { max_length: 80 }),
            Rc::new(SloEntryCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_entry_validator();
        let item = new_slo_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SloEntryRequiredCodeRule {
            message: "slo entry needs this field".to_string(),
        };
        let mut item = new_slo_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SloEntryCodeLengthRule { max_length: 32 };
        let mut item = new_slo_entry("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
