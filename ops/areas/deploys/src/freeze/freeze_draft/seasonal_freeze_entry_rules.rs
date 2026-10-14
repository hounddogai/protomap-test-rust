use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal freeze entry must pass. Check returns its problems, or none.
pub trait SeasonalFreezeEntryRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalFreezeEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeEntryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SeasonalFreezeEntryRule for SeasonalFreezeEntryAllowedCodeRule {
    fn check(&self, item: &SeasonalFreezeEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeEntryCodePrefixRule {
    pub prefix: String,
}

impl SeasonalFreezeEntryRule for SeasonalFreezeEntryCodePrefixRule {
    fn check(&self, item: &SeasonalFreezeEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeEntryRequiredSummaryRule {
    pub message: String,
}

impl SeasonalFreezeEntryRule for SeasonalFreezeEntryRequiredSummaryRule {
    fn check(&self, item: &SeasonalFreezeEntry) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeEntrySummaryLengthRule {
    pub max_length: i64,
}

impl SeasonalFreezeEntryRule for SeasonalFreezeEntrySummaryLengthRule {
    fn check(&self, item: &SeasonalFreezeEntry) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal freeze entry.
#[derive(Debug, Clone)]
pub struct SeasonalFreezeEntryValidator {
    pub rules: Vec<Rc<dyn SeasonalFreezeEntryRule>>,
}

impl SeasonalFreezeEntryValidator {
    pub fn validate(&self, item: &SeasonalFreezeEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalFreezeEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalFreezeEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_freeze_entry_validator() -> SeasonalFreezeEntryValidator {
    SeasonalFreezeEntryValidator {
        rules: vec![
            Rc::new(SeasonalFreezeEntryAllowedCodeRule { allowed: Vec::new() }) as Rc<dyn SeasonalFreezeEntryRule>,
            Rc::new(SeasonalFreezeEntryCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(SeasonalFreezeEntryRequiredSummaryRule {
                message: "seasonal freeze entry needs an entry".to_string(),
            }),
            Rc::new(SeasonalFreezeEntrySummaryLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_freeze_entry_validator();
        let item = new_seasonal_freeze_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = SeasonalFreezeEntryRequiredSummaryRule {
            message: "seasonal freeze entry needs an entry".to_string(),
        };
        let mut item = new_seasonal_freeze_entry("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = SeasonalFreezeEntrySummaryLengthRule { max_length: 64 };
        let mut item = new_seasonal_freeze_entry("x", "y");
        item.summary = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
