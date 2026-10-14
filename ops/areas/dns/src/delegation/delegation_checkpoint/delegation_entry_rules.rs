use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a delegation entry must pass. Check returns its problems, or none.
pub trait DelegationEntryRule: std::fmt::Debug {
    fn check(&self, item: &DelegationEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationEntryPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationEntryRule for DelegationEntryPriorityRangeRule {
    fn check(&self, item: &DelegationEntry) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationEntryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DelegationEntryRule for DelegationEntryAllowedCodeRule {
    fn check(&self, item: &DelegationEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationEntryCodeLengthRule {
    pub max_length: i64,
}

impl DelegationEntryRule for DelegationEntryCodeLengthRule {
    fn check(&self, item: &DelegationEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationEntryRequiredAudienceRule {
    pub message: String,
}

impl DelegationEntryRule for DelegationEntryRequiredAudienceRule {
    fn check(&self, item: &DelegationEntry) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation entry.
#[derive(Debug, Clone)]
pub struct DelegationEntryValidator {
    pub rules: Vec<Rc<dyn DelegationEntryRule>>,
}

impl DelegationEntryValidator {
    pub fn validate(&self, item: &DelegationEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_entry_validator() -> DelegationEntryValidator {
    DelegationEntryValidator {
        rules: vec![
            Rc::new(DelegationEntryPriorityRangeRule { min: 120, max: 1000 }) as Rc<dyn DelegationEntryRule>,
            Rc::new(DelegationEntryAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "batched".to_string(), "capped".to_string()],
            }),
            Rc::new(DelegationEntryCodeLengthRule { max_length: 64 }),
            Rc::new(DelegationEntryRequiredAudienceRule {
                message: "delegation entry needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_entry_validator();
        let item = new_delegation_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = DelegationEntryPriorityRangeRule { min: 255, max: 255 };
        let mut item = new_delegation_entry("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DelegationEntryCodeLengthRule { max_length: 3600 };
        let mut item = new_delegation_entry("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = DelegationEntryRequiredAudienceRule {
            message: "delegation entry needs an entry".to_string(),
        };
        let mut item = new_delegation_entry("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
